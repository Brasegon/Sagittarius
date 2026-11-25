//! The platform traits, implemented and used from outside the crate.
//!
//! A trait has no behaviour of its own, so there is nothing to test inside it. These tests check
//! its shape instead: they implement each trait with a small fake, then use it the way the
//! flight software will. If a signature differs from what was agreed, this file does not compile.
//!
//! A file in `tests/` is compiled as a separate crate, like `sgr-flight-linux`: it sees only
//! what is `pub`.

use sgr_flight_core::platform::{BusError, Clock, CycleSync, EquipmentBus};

// --- Clock ---

/// A clock that gives the tick it was told, as the lockstep platform will do with the tick of
/// the simulator.
struct FakeClock {
    current: u32,
}

impl Clock for FakeClock {
    fn tick(&self) -> u32 {
        self.current
    }
}

/// On-board time in milliseconds. It works with any clock, because it only knows the trait.
fn on_board_millis<C: Clock>(clock: &C) -> u64 {
    u64::from(clock.tick()) * 100
}

#[test]
fn clock_gives_the_current_tick() {
    let mut clock = FakeClock { current: 0 };
    assert_eq!(clock.tick(), 0);

    // The simulator announces the next tick: the clock follows.
    clock.current = 1;
    assert_eq!(clock.tick(), 1);
}

#[test]
fn clock_is_used_through_the_trait() {
    // 1 tick = 100 ms (ICD, "Conventions").
    assert_eq!(on_board_millis(&FakeClock { current: 0 }), 0);
    assert_eq!(on_board_millis(&FakeClock { current: 74_565 }), 7_456_500);
    assert_eq!(
        on_board_millis(&FakeClock { current: u32::MAX }),
        429_496_729_500
    );
}

// --- EquipmentBus ---

/// Address of the only equipment on the fake bus.
const ADDRESS: u8 = 0x10;
/// The measure it answers with: five bytes.
const MEASURE: [u8; 5] = [0x6D, 0x60, 0xFD, 0xF3, 0x00];

/// A bus with a single equipment, which understands two requests:
/// - `[0x01]`: it answers with `MEASURE`;
/// - `[0x02, value]`: it stores `value` and answers with an empty response.
///
/// It stays silent for anything else.
struct FakeBus {
    stored: u8,
    exchanges: u32,
}

impl FakeBus {
    fn new() -> Self {
        Self {
            stored: 0,
            exchanges: 0,
        }
    }
}

impl EquipmentBus for FakeBus {
    fn transfer(
        &mut self,
        address: u8,
        request: &[u8],
        response: &mut [u8],
    ) -> Result<usize, BusError> {
        self.exchanges += 1;

        if address != ADDRESS {
            return Err(BusError::NoResponse);
        }

        match *request {
            [0x01] => {
                if response.len() < MEASURE.len() {
                    return Err(BusError::ResponseTooLong);
                }
                response[..MEASURE.len()].copy_from_slice(&MEASURE);
                Ok(MEASURE.len())
            }
            [0x02, value] => {
                self.stored = value;
                Ok(0)
            }
            _ => Err(BusError::NoResponse),
        }
    }
}

/// A bus on which nothing answers. It has no field: a struct can be empty.
struct DeadBus;

impl EquipmentBus for DeadBus {
    fn transfer(
        &mut self,
        _address: u8,
        _request: &[u8],
        _response: &mut [u8],
    ) -> Result<usize, BusError> {
        Err(BusError::NoResponse)
    }
}

/// What a driver does: one exchange, then it keeps the bytes that came back. It works with any
/// bus, because it only knows the trait.
fn read_measure<B: EquipmentBus>(bus: &mut B) -> Result<[u8; 5], BusError> {
    let mut buffer = [0u8; 5];
    bus.transfer(ADDRESS, &[0x01], &mut buffer)?;
    Ok(buffer)
}

#[test]
fn bus_copies_the_response_and_returns_its_length() {
    let mut bus = FakeBus::new();
    // Larger than the response: the returned length says how many bytes are meaningful.
    let mut buffer = [0xAA; 8];

    assert_eq!(bus.transfer(ADDRESS, &[0x01], &mut buffer), Ok(5));
    assert_eq!(&buffer[..5], &MEASURE[..]);
    assert_eq!(&buffer[5..], &[0xAA; 3][..]);
    assert_eq!(bus.exchanges, 1);
}

#[test]
fn bus_tells_an_empty_response_from_silence() {
    let mut bus = FakeBus::new();
    let mut buffer = [0u8; 8];

    // The equipment obeyed and has nothing to say: an empty response, not an error.
    assert_eq!(bus.transfer(ADDRESS, &[0x02, 0x01], &mut buffer), Ok(0));
    assert_eq!(bus.stored, 0x01);

    // The equipment does not understand the request, or nobody has this address: silence.
    assert_eq!(
        bus.transfer(ADDRESS, &[0x03], &mut buffer),
        Err(BusError::NoResponse)
    );
    assert_eq!(
        bus.transfer(0x11, &[0x01], &mut buffer),
        Err(BusError::NoResponse)
    );
}

#[test]
fn bus_refuses_a_response_that_does_not_fit() {
    let mut bus = FakeBus::new();

    let mut too_small = [0u8; 4];
    assert_eq!(
        bus.transfer(ADDRESS, &[0x01], &mut too_small),
        Err(BusError::ResponseTooLong)
    );

    // An empty buffer is enough for an empty response.
    assert_eq!(bus.transfer(ADDRESS, &[0x02, 0x01], &mut []), Ok(0));
}

#[test]
fn driver_code_works_with_any_bus() {
    assert_eq!(read_measure(&mut FakeBus::new()), Ok(MEASURE));
    assert_eq!(read_measure(&mut DeadBus), Err(BusError::NoResponse));
}

#[test]
fn bus_error_is_a_plain_value() {
    // Like `DecodeError`: it can be copied, compared and printed.
    let error = BusError::NoResponse;
    let copy = error;
    assert_eq!(error, copy);
    assert_ne!(BusError::NoResponse, BusError::ResponseTooLong);
    assert_eq!(format!("{error:?}"), "NoResponse");
}

// --- CycleSync ---

/// A platform reduced to its cadence, with its clock: each wait starts the next tick, as the
/// `Tick` frame of the simulator will.
struct FakeCadence {
    current: u32,
    cycles_done: u32,
}

impl CycleSync for FakeCadence {
    fn wait_tick(&mut self) {
        self.current += 1;
    }

    fn cycle_done(&mut self) {
        self.cycles_done += 1;
    }
}

impl Clock for FakeCadence {
    fn tick(&self) -> u32 {
        self.current
    }
}

/// A cycle with nothing in it. `P: CycleSync + Clock` asks for a type that implements both
/// traits: the same platform gives the cadence and the time.
fn empty_cycle<P: CycleSync + Clock>(platform: &mut P) -> u32 {
    platform.wait_tick();
    let tick = platform.tick();
    platform.cycle_done();
    tick
}

#[test]
fn cycle_sync_frames_each_cycle() {
    let mut platform = FakeCadence {
        current: 0,
        cycles_done: 0,
    };

    platform.wait_tick();
    platform.cycle_done();
    platform.wait_tick();
    platform.cycle_done();

    assert_eq!(platform.cycles_done, 2);
}

#[test]
fn tick_of_a_cycle_is_read_through_the_clock() {
    // `wait_tick` returns nothing: the tick has a single source, the clock.
    let mut platform = FakeCadence {
        current: 41,
        cycles_done: 0,
    };

    assert_eq!(empty_cycle(&mut platform), 42);
    assert_eq!(empty_cycle(&mut platform), 43);
    assert_eq!(platform.cycles_done, 2);
}
