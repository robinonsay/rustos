# sim_harness — Interface Definitions (§4)

**Parent:** [`design.md`](./design.md) (index — IEEE 1016 §§1–3, 5–11).
**Scope of this file:** §4 only — S_define entries and the companion-code
contract. Split out for the 500-line cap; all other sections remain in
`design.md`.

---

<!-- @{"design": ["SW-REQ-SIM-HARN-001", "SW-REQ-SIM-HARN-002", "SW-REQ-SIM-HARN-003", "SW-REQ-SIM-HARN-004", "SW-REQ-SIM-HARN-005", "SW-REQ-SIM-HARN-007", "SW-REQ-SIM-HARN-008", "SW-REQ-SIM-HARN-009", "SW-REQ-SIM-HARN-010"]} -->
## 4. Interface Definitions — S_define Entries and Companion Code

### 4.1 S_define structure (declarative skeleton)

```text
/* sim/sim_harness/S_define — illustrative skeleton */
##include "sim_scenario/sim_scenario.hpp"
##include "sim_dynamics/sim_dynamics.hpp"
##include "sim_sensors/sim_sensors.hpp"
##include "sim_harness/sim_harness.hpp"

sim_object {                                      // harness (declared FIRST — owns args)
    juno::sim_harness::SIM_HARNESS_T tImpl;
    juno::sim_harness::HARNESS_ARGS_T tArgs;      // populated from argc/argv before sched start
    (initialization)   tImpl.Init(tArgs, scen.tScenario, dyn.tImpl, sens.tImpl);
    (1e-3, scheduled)  tImpl.TickFsw();           // drives FSW POSIX scheduler
    (1e-3, scheduled)  tImpl.CaptureBus();        // siphons broker messages
    (1e-3, scheduled)  tImpl.WriteTruthRow();     // SW-REQ-SIM-HARN-009
    (shutdown)         tImpl.FinalizeArtifacts();
} harness;
sim_object {                                      // scen — passive POD holder
    juno::sim_scenario::SIM_SCENARIO_T tScenario; // populated by harness.Init step 1.3
    /* No (initialization) job here. LoadScenario runs INSIDE harness.Init
     * (see §4.3 step 1.3) so the FSW composition root, sensor cfg
     * transcoding, and dynamics cfg transcoding all observe a fully-populated
     * tScenario. Earlier draft had `(initialization) tScenario =
     * LoadScenario(harness.tArgs.acScenarioPath).tOk;` which deadlocked
     * because harness.Init was already reading scen.tScenario before this
     * job ran (declaration order is harness → scen → dyn → sens). */
} scen;
sim_object {                                      // dyn
    SIM_DYNAMICS tImpl;                           // sim-only; not in juno::* namespace
    SIM_DYN_TRUTH_T tTruth;                       // POD — see sim_dynamics §6.1
    (initialization) tImpl.initialize();          // pulled from scen.tScenario at Init time
    (derivative)     tImpl.derivative();
    (integration)    trick_ret = tImpl.integration();
} dyn;
sim_object {                                      // sens
    sim_sensors::SimSensors tImpl;                // sim-only; not in juno::* namespace
    (initialization)        tImpl.Init(harness.tImpl._tSimSensorCfg);  // transcoded from flat scen.tScenario at harness.Init time
    (5e-3, scheduled)       tImpl.Step(harness.tImpl.NowUs(), dyn.tTruth);
} sens;
connect { &sens.tImpl.ImuRaw()   -> harness.tImpl._ptImuDriverInput;
          &sens.tImpl.BaroRegs() -> harness.tImpl._ptBaroDriverInput; }
/* GPS is NOT bound by address. SimSensors has no GPS accessor; it pushes
 * NMEA bytes through a sink callback installed inside harness.Init. The
 * harness owns a pseudo-terminal pair (openpty(3)) — the FSW device_lib::posix
 * UART receives the slave fd via its New() factory exactly as the host-test
 * pty fixture does (device/design.md §4.3 / equivalence table). The harness
 * retains the master fd on SIM_HARNESS_T._iGpsPtyMasterFd and installs
 *   sens.tImpl.SetGpsUartSink({pfnWrite=&SimHarness::WriteGpsMasterFd,
 *                              pvCtx=&harness.tImpl});
 * whose body is ::write(_iGpsPtyMasterFd, pcBuf, zLen). See §4.4.1 GPS row. */
integrate (1e-3) dyn;                             // fixed-step RK4 at 1 ms
```

Exact Trick keywords (`integ_loop`, etc.) deferred to the implementation
worker; §4.2–§4.4 specify the contractual behaviour the S_define must produce.

**Declaration order rationale:** `harness` is declared **first** so its
`(initialization) tImpl.Init(...)` job runs before any other sim-object's
`(initialization)` job. `harness.Init` is the unique site that loads the
scenario (`LoadScenario(tArgs.acScenarioPath)` — see §4.3 step 1.3),
populates `scen.tScenario`, and produces the two transcoded cfg PODs that
`dyn.initialize` and `sens.Init` consume. `tArgs` itself is a POD whose
fields are populated from argc/argv by `main.cpp` *before* Trick's
`initialization` phase fires, so `harness.Init` has valid args from its
first instruction. Trick `connect` blocks bind by address and are evaluated
after all sim-objects are constructed, so they are unaffected by declaration
order.

### 4.2 sim_jobs (scheduler wiring)

| Job | Period | Source object | Purpose |
|-----|--------|--------------|---------|
| `dyn.derivative` | derivative | `dyn` | EOM derivatives at each RK4 sub-evaluation. |
| `dyn.integration` | 1 ms; one RK4 step per Trick tick (4 derivative evaluations) | `dyn` | Fixed-step RK4 integrator (per `sim_dynamics` §4.4). |
| `sens.Step` (IMU rate-gate) | 5 ms native | `sens` | Synthetic IMU at 200 Hz (`SW-REQ-SYS-005`). |
| `sens.Step` (baro rate-gate) | 50 ms native | `sens` | Synthetic baro at 20 Hz (`SW-REQ-SYS-008`). |
| `sens.Step` (GPS rate-gate) | 200 ms native | `sens` | Synthetic GPS at 5 Hz (`SW-REQ-SYS-009`). |
| `harness.TickFsw` | 1 ms | `harness` | Advances FSW `sch_lib` by one Trick base tick (drives apps on their TDM boundaries). |
| `harness.CaptureBus` | 1 ms | `harness` | Subscribes to all broker messages; appends to in-RAM ring; flushed by `WriteTruthRow`. |
| `harness.WriteTruthRow` | 1 ms | `harness` | Writes one truth row + last NAV_STATE row to comparison artifact (`SW-REQ-SIM-HARN-009`). |

`sens.Step` is invoked at the dynamics tick (1 ms); the per-sensor publish
gate inside `SimSensors::Step` rate-divides to 200 Hz / 20 Hz / 5 Hz per
`sim_sensors` §5.2. The harness does **not** schedule three separate
`SampleX` jobs.

### 4.3 Object instantiation order

Trick `initialization` jobs run in declared object order. The harness imposes
this order. **Note on Init-ordering deadlock fix:** an earlier draft made
`harness.Init(tArgs, scen.tScenario, ...)` run *before* `scen.LoadScenario`
populated `scen.tScenario`, leaving every downstream consumer with a
zero-initialised scenario POD. The fix below relocates `LoadScenario` to be
the **first** action inside `harness.Init`, so the scenario is always loaded
before any composition step that depends on `tScenario`. The `scen`
sim-object thereafter only stores a const reference / copy that the harness
already produced — it has no `(initialization)` job of its own that races
against `harness.Init`.

1. `harness` — `Init(tArgs, scen.tScenario, dyn.tImpl, sens.tImpl)` performs,
   in this order:
   1. **Parse / validate `tArgs`** (`acScenarioPath`, `acOutDir`, `u64Seed`,
      `dStopS`). `tArgs` is a POD already populated from argc/argv by
      `main.cpp` before Trick's `initialization` phase runs (§4.1
      "Declaration order rationale").
   2. **Create the run output directory** under `tArgs.acOutDir`
      (`SW-REQ-SIM-HARN-010`); on failure return `JUNO_STATUS_WRITE_ERROR`
      (`conventions.md` §4.8).
   3. **Load the scenario early** — call
      `juno::sim_scenario::LoadScenario(tArgs.acScenarioPath)` and write the
      returned flat `SIM_SCENARIO_T` POD into `scen.tScenario`
      (`SW-REQ-SIM-HARN-005`). Errors abort `Init` before any FSW or
      sim-side composition runs (no zero-initialised scenario can leak
      downstream). Sensor and dynamics parameters are top-level fields of
      `SIM_SCENARIO_T` per `sim_scenario` design — there are **no nested cfg
      substructures**.
   4. **Transcode** flat scenario fields into the two locally-owned cfg
      structs on `SIM_HARNESS_T`: `_tSimSensorCfg` (`SIM_SENSOR_CFG_T`, per
      `sim_sensors` §4.4) from fields like `dImuAccelSigmaMps2`,
      `dBaroSigmaPa`, `dGpsHorizSigmaM`; and `_tSimDynInitCfg`
      (`SIM_DYNAMICS::INIT_CFG_T`, per `sim_dynamics` §4.2 / §4.4) from the
      mass / inertia / thrust-curve / launch-site fields. Downstream `Init`
      / `initialize` jobs consume these by const-ref.
   5. **Open the GPS pty pair** — `openpty(&_iGpsPtyMasterFd,
      &_iGpsPtySlaveFd, ...)` (`device/design.md` §4.3 / equivalence table).
      The slave fd is handed to the FSW `device_lib::posix` UART
      `New()` factory in step 6; the master fd stays on `SIM_HARNESS_T` for
      the GPS push sink (§4.4 GPS row).
   6. **Invoke the FSW composition root** — the same per-lib `New()` factory
      sequence as `apps/main_posix.cpp` (per-driver buffer-injection seams
      from §4.4.1) plus the per-app free-function setup
      `juno::<app>::<App>AppInit(tApp, &libRoot, &broker, tTime)` for every
      FSW app, which aggregate-initialises each app's
      `juno::app::APP_ROOT_T tRoot` with its
      `juno::app::APP_API_T { OnStart, OnProcess, OnExit }` vtable and
      registers the `APP_ROOT_T*` into the FSW
      `juno::sch::SCH_ROOT_T<8, 200>` schedule table (`system_design.md`
      §8.1). The harness never calls `OnStart`/`OnProcess`/`OnExit` directly
      — the FSW scheduler dispatches lifecycle hooks via the registered
      vtable pointer.
   7. **Bind Trick time** — call
      `juno::time::TimeInit(tTime, tTrickTimeApi, /*pfcnFailureHandler=*/nullptr,
      /*pvUserData=*/nullptr)` to point the FSW's `juno::time::TIME_ROOT_T`
      at the harness-side `TIME_API_T` aggregate (§4.4). The previous POSIX
      `tPosixTimeApi` would have been bound here in the flight POSIX build;
      only this single `TIME_API_T` instance differs across targets.
   8. **Install the GPS UART sink** —
      `sens.tImpl.SetGpsUartSink({pfnWrite = &SimHarness::WriteGpsMasterFd,
      pvCtx = this})`; the body of `WriteGpsMasterFd` is
      `::write(_iGpsPtyMasterFd, pcBuf, zLen)` (§4.4.1 GPS row).
   9. **Subscribe** the bus-capture sink to every `JUNO_MSG_*` type the
      broker carries.
2. `dyn` — `(initialization) SIM_DYNAMICS::initialize(harness.tImpl._tSimDynInitCfg)`
   reads the transcoded mass / inertia / thrust-curve / launch-site cfg
   (per `sim_dynamics` §4.2 / §4.4) — **never** indexes a dynamics-cfg
   substructure on `scen.tScenario` (the scenario POD is flat per
   `sim_scenario` design; no such substructure exists).
3. `sens` — `(initialization) SimSensors::Init(harness.tImpl._tSimSensorCfg)`
   consumes the transcoded noise / bias / dropout `SIM_SENSOR_CFG_T` by
   const-ref (per `sim_sensors` §4.4) — **never** indexes a sensor-cfg
   substructure on `scen.tScenario`.
4. `scen` is declared as a passive POD-holder sim-object: `scen.tScenario`
   is written by `harness.Init` (step 1.3) and read by the harness during
   transcoding; no `(initialization)` job on `scen` itself runs `LoadScenario`
   (the previous draft's deadlock).

Trick's declaration-order initialization rule plus the rule that Trick
`connect` blocks bind by address (evaluated after all sim-objects are
constructed) makes this order safe: `harness.Init` runs first, populates
`scen.tScenario` and the two transcoded cfg PODs, then `dyn` and `sens`
initialize against those PODs; `connect` resolves sensor-output addresses
into harness driver-input pointers after every sim-object is constructed.

### 4.4 Variable bindings — truth → sensor inputs → FSW POSIX driver inputs → artifacts

The harness companion exposes the data plane as POD members on
`SIM_HARNESS_T` so Trick can reach them via `connect` blocks:

| Bind direction | Source | Sink | Type |
|---------------|--------|------|------|
| Dynamics → sensors | `dyn.tTruth` | `sens.tImpl.Step(now, truth)` parameter | `SIM_DYN_TRUTH_T` (read-only by sensors) |
| Sensors → driver inputs (IMU, baro pull) | `sens.tImpl.ImuRaw()`, `sens.tImpl.BaroRegs()` | `harness._ptImuDriverInput`, `_ptBaroDriverInput` | Pointers to `SIM_SENSORS_RAW_T` (IMU register image) and `SIM_BARO_REGS_T` (MPL3115A2 register image; see `sim_sensors` §4.2). GPS is **not** bound by address — see GPS push row below. |
| Sensors → driver inputs (GPS push) | `sim_sensors gps_model` (5 Hz) | `device_lib::posix` UART RX path (via openpty master fd write) | The harness owns a pseudo-terminal pair created by `openpty(3)` during `harness.Init`. The slave fd is handed to the FSW `device_lib::posix::DEVICE_LIB_IMPL_T<2048>::New(...)` factory, exactly as the host-test pty fixture does (`device/design.md` §4.3 / equivalence table — "Tests inject NMEA bytes by `write()` to the master end of the pty; the impl reads from the slave end exactly as it would from a real serial line"). The master fd is retained on `SIM_HARNESS_T._iGpsPtyMasterFd`. Harness installs `SimSensors::SetGpsUartSink({pfnWrite = &SimHarness::WriteGpsMasterFd, pvCtx = &harness.tImpl})` at `harness.Init`; the body is `::write(_iGpsPtyMasterFd, pcBuf, zLen)`. `gps_model` then pushes formatted NMEA bytes through the callback at 5 Hz; `gps_lib::posix` reads them via its normal `Read()` path. No GPS output struct, no GPS-by-address binding, no fabricated `device_lib::posix::Inject` symbol, and no harness-side per-tick GPS poll. |
| Driver inputs → FSW | `harness._pt*DriverInput` | FSW POSIX driver `Sample()` / `Read()` reads | Driver impls dereference a sim-injected pointer instead of `read()`-ing a file descriptor. Pointers are wired into each driver via its `New()` factory (§4.4.1). |
| Trick time → FSW time | Trick sim time (`double` s) | FSW `juno::time::TIME_ROOT_T` aggregate at composition | `JUNO_TIMESTAMP_T` (POD `{iSeconds, iSubSeconds}`). The harness defines `static const juno::time::TIME_API_T tTrickTimeApi{TrickNow, TrickSleepTo, TrickSleep};` in `sim/sim_harness/src/time_trick_source.cpp` and the FSW composition root in the harness build calls `juno::time::TimeInit(tTime, tTrickTimeApi, ...)` instead of binding the POSIX `tPosixTimeApi`. Consumers that need microseconds derive them via `tTime.TimestampToMicros(tNow).tOk` (the LibJuno-canonical member function on `TIME_ROOT_T`). **No FSW source modified** — only the `TIME_API_T` instance bound at composition changes. **No `JUNO_TIME_PROVIDER_T` callback** (Option A — Chair 2026-05-03). |
| Bus → artifact | broker subscriber slots | `harness.WriteTruthRow`, telemetry / mlog sinks | Bus messages captured per-tick into per-run output files. |

**FSW time injection mechanism (canonical, Option A — Chair 2026-05-03):** there is no `JUNO_TIME_PROVIDER_T` callback and no FT1 `TIME_LIB_IMPL_T` factory. The injection seam is the LibJuno-published `juno::time::TimeInit(tTime, tApi, pfcnFailureHandler, pvUserData)` — replace the `tApi` argument and the entire time vtable changes:

```cpp
// sim/sim_harness/src/time_trick_source.cpp  (illustrative)
//
// File-scope aggregate-init of the canonical juno::time::TIME_API_T vtable.
// No JUNO_TIME_PROVIDER_T callback, no TIME_LIB_IMPL_T::New(...) factory,
// no juno_time.tSimUs variable-server pull-model — Option A replaces the
// entire vtable at composition.
// RESULT_T<T> is published from juno/macros.h; the time_api.hpp header uses
// it unqualified inside namespace juno::time. We mirror that here.
namespace juno { namespace time {
RESULT_T<JUNO_TIMESTAMP_T>
TrickNow(const TIME_ROOT_T &tTime) noexcept
{
    // Trick API (header sim_services/include/sim_services/exec_proto.h):
    //   double exec_get_sim_time(void);   // seconds, monotonic in sim time
    // Forward to LibJuno's canonical double-seconds → timestamp helper
    // (TIME_ROOT_T::DoubleToTimestamp, libjuno/include/juno/time/time_api.hpp
    // L426 onwards), which already validates non-negative and in-range.
    return tTime.DoubleToTimestamp(::exec_get_sim_time());
}
}} // namespace juno::time
namespace juno { namespace time {
JUNO_STATUS_T TrickSleepTo(const TIME_ROOT_T &, JUNO_TIMESTAMP_T) noexcept
{ return JUNO_STATUS_SUCCESS; } // simulated time advances under Trick control
JUNO_STATUS_T TrickSleep  (const TIME_ROOT_T &, JUNO_TIMESTAMP_T) noexcept
{ return JUNO_STATUS_SUCCESS; }
}} // namespace juno::time

// Aggregate-init at file scope; outlives every TIME_ROOT_T it is bound into.
static const juno::time::TIME_API_T tTrickTimeApi{
    juno::time::TrickNow,
    juno::time::TrickSleepTo,
    juno::time::TrickSleep
};

// Harness composition root (inside SIM_HARNESS_T::Init step 1.7) binds the
// Trick API into the FSW's TIME_ROOT_T:
juno::time::TIME_ROOT_T tTime;
JUNO_ASSERT_SUCCESS(
    juno::time::TimeInit(tTime, tTrickTimeApi,
                          /*pfcnFailureHandler=*/nullptr,
                          /*pvUserData=*/nullptr),
    return /*halt*/);
```

Consumers obtain microseconds as
`tTime.TimestampToMicros(tTime.ptApi->Now(tTime).tOk).tOk` — both
`TimestampToMicros` and `Now` are member / vtable-dispatched calls on the
same `TIME_ROOT_T`, exactly as on the flight POSIX and Pico2 builds. The
flight POSIX build and the Pico2 build bind their respective
`tPosixTimeApi` / `tPico2TimeApi` instances at the same
`juno::time::TimeInit` call site — only the `TIME_API_T` instance differs
across targets. The previous `juno_time.tSimUs` variable-server pull model,
the speculative `-fno-builtin` linker shim, and any `JUNO_TIME_PROVIDER_T` /
`TIME_LIB_IMPL_T::New(pfcn, ...)` factory variant are all **superseded** by
Option A and must not appear in any sim_harness artefact.

#### 4.4.1 Per-driver `New()`-time buffer-injection seams

Each FSW POSIX driver impl already exposes a `New()` parameter for its
sim-side input source — this is the seam the harness uses to inject the
`SimSensors` outputs without modifying any FSW source
(`SW-REQ-SIM-HARN-004`). The seams are:

| Driver | `New()` injection parameter | Source bound by harness | Reference |
|--------|-----------------------------|--------------------------|-----------|
| `imu_lib::posix` | `const SIM_SENSORS_RAW_T *` (held as `pvPlatform`) | Address of `sens.tImpl.ImuRaw()` | `imu/design.md` §4.3 |
| `baro_lib::posix` | `BARO_LIB_BUS_T tBus` — `WriteReg` / `ReadReg` callback pair | Harness-supplied shim callbacks that map MPL3115A2 register reads/writes onto fields of `sens.tImpl.BaroRegs()` (`u8Status`, `u8OutP{Msb,Csb,Lsb}`, `u8OutT{Msb,Lsb}`, `u8WhoAmI = 0xC4`, `bIoOk`). The byte-level Q18.2 / Q12.4 decode lives inside `baro_lib`, not in the sim — the harness shim transports raw register bytes only. | `baro/design.md` §4.1, `sim_sensors` §4.2 |
| `gps_lib::posix` | `juno::device::DEVICE_LIB_ROOT_T<2048> *ptDevice` | Harness creates an `openpty(3)` master/slave pair during `harness.Init`. The slave fd is passed into `juno::device::DEVICE_LIB_IMPL_T<2048>::New(...)` (the canonical POSIX impl per `device/design.md` §4.3 / "POSIX/Pico2 functional equivalence" table — `iFd` opened on the pty slave end), and the resulting `DEVICE_LIB_ROOT_T<2048>*` is then passed to `gps_lib::posix::New(...)` exactly as in the flight POSIX build. The master fd is retained on `SIM_HARNESS_T._iGpsPtyMasterFd`. The harness then installs `sens.tImpl.SetGpsUartSink({pfnWrite = &SimHarness::WriteGpsMasterFd, pvCtx = &harness.tImpl})`; the body is `::write(_iGpsPtyMasterFd, pcBuf, zLen)`. The `sim_sensors` `gps_model` thereafter **pushes** formatted NMEA bytes (5 Hz cadence) through the sink → master fd → kernel pty → slave fd → `device_lib::posix::ReadBytes` RX ring → `gps_lib::posix` `Read()`. No `acNmeaSentence` field; no per-tick poll on the harness side; no fabricated `device_lib::posix::Inject` symbol. | `gps/design.md` §4.1, `device/design.md` §4.3, `sim_sensors` §4.4 |
| `lora_lib::posix` | platform transport at `New()` | Harness-supplied byte sink → `telemetry.bin` writer | (`lora` POSIX impl) |
| `sd_lib::posix` | platform transport at `New()` | Harness-supplied directory under `<out_dir>/sd_log/` | (`sd` POSIX impl) |

Every binding above goes through the driver's existing `New()` argument
list — no driver source is patched, no symbol is interposed. The harness
companion holds the shim callback bodies (e.g., the
`baro_lib::BARO_LIB_BUS_T::ReadReg` closure that maps an MPL3115A2 register
read onto the corresponding field of the latest `sens.tImpl.BaroRegs()`
register image, and `SimHarness::WriteGpsMasterFd` whose body is
`::write(_iGpsPtyMasterFd, pcBuf, zLen)` against the openpty master fd
described in the GPS row above) in `src/sim_harness.cpp` and passes them
into each driver factory inside `Init()`.

### 4.5 Companion contract — `juno::sim_harness::SIM_HARNESS_T`

`HARNESS_ARGS_T` is a POD holding the parsed CLI: `acScenarioPath[kPathMax]`,
`acOutDir[kPathMax]`, `u64Seed`, `dStopS`. It is populated by `main.cpp`
before Trick's `initialization` jobs fire (§4.3).

| Member | Signature |
|--------|-----------|
| `Init` | `JUNO_STATUS_T Init(const HARNESS_ARGS_T &tArgs, const SIM_SCENARIO_T &tScen, SIM_DYNAMICS &tDyn, sim_sensors::SimSensors &tSens) noexcept` |
| | **Preconditions:** `tArgs` populated by argv parsing; `tScen` is the *out-parameter* whose storage lives on the `scen` sim-object — `Init` writes it via `LoadScenario` (step 1.3) and the caller observes it filled when `Init` returns; `tDyn`/`tSens` constructed (Trick will run their `initialize()` / `Init()` jobs after harness has transcoded cfgs). |
| | **Postconditions:** output dir created (`SW-REQ-SIM-HARN-010`); `tScen` populated by `LoadScenario(tArgs.acScenarioPath)` (`SW-REQ-SIM-HARN-005`); flat `tScen` fields **transcoded** into `_tSimSensorCfg` (`SIM_SENSOR_CFG_T`, per `sim_sensors` §4.4) and `_tSimDynInitCfg` (per `sim_dynamics` §4.4) for downstream `Init`/`initialize` to consume by const-ref. **Sensor-noise transcoding narrows `double` → `float`**: every `double dImu*Sigma*` / `dGps*Sigma*` / bias / drift-rate field on flat `SIM_SCENARIO_T` is `static_cast<float>(...)` into the corresponding `float fImu*Noise*Sigma*` / `fGps*Sigma*` field of `SIM_SENSOR_CFG_T` because the sensor models run in single precision (the precision floor on Trick-emulated sensor noise is well above `float` epsilon at FT1 sigma magnitudes ~10⁻³ m/s² and ~10⁻⁴ rad/s; double precision adds no observable accuracy). The narrowing is intentional and centralised in this transcoding step; openpty master/slave pair created (master fd retained on `_iGpsPtyMasterFd`; slave fd handed to `device_lib::posix::DEVICE_LIB_IMPL_T<2048>::New(...)`); FSW composition root invoked — every per-lib `New()` runs and every FSW app's free-function setup `juno::<app>::<App>AppInit(tApp, &libRoot, &broker, tTime)` aggregate-initialises its `juno::app::APP_ROOT_T` with the `juno::app::APP_API_T { OnStart, OnProcess, OnExit }` vtable and registers the `APP_ROOT_T*` into the FSW `juno::sch::SCH_ROOT_T<8, 200>` (`system_design.md` §8.1); `tSens.SetGpsUartSink({pfnWrite=&SimHarness::WriteGpsMasterFd, pvCtx=&harness.tImpl})` installed (§4.4.1 GPS row); the FSW `juno::time::TIME_ROOT_T` initialized via `juno::time::TimeInit(tTime, tTrickTimeApi, /*pfcnFailureHandler=*/nullptr, /*pvUserData=*/nullptr)` and returned `JUNO_STATUS_SUCCESS` (Option A — no `JUNO_TIME_PROVIDER_T` callback, no `TIME_LIB_IMPL_T::New(pfcn, ...)` factory); per-driver buffer-injection pointers wired for IMU and baro (§4.4.1); bus capture subscribed. |
| | **Errors:** `JUNO_STATUS_WRITE_ERROR` if output dir cannot be created (canonical write-path code per `conventions.md` §4.8); failure handler called diagnostically (no control-flow change, `conventions.md` §4.3). |
| | **Thread safety:** Single-threaded inside Trick sim-object; not thread-safe across threads. Trick is single-threaded for this harness (§3, `system_design.md` §3). |
| `NowUs` | `JUNO_TIME_US_T NowUs() const noexcept` |
| | **Preconditions:** none (callable before `Init` since it only reads Trick's clock). |
| | **Postconditions:** Convenience accessor for harness-only consumers (artifact-row timestamping when no `TIME_ROOT_T` is in scope) that returns `exec_get_sim_time()` rounded to `JUNO_TIME_US_T` microseconds. Monotonic. **Not** in the FSW time-injection path — the FSW's `TIME_API_T::Now` is `TrickNow`, which consumes `exec_get_sim_time()` directly via `tTime.DoubleToTimestamp(...)` (§4.4) — so `NowUs` is reserved for harness companion artifact code only. |
| | **Errors:** none. |
| | **Thread safety:** Single-threaded inside Trick sim-object; not thread-safe across threads. |
| `TickFsw` | `void TickFsw() noexcept` |
| | **Preconditions:** `Init` succeeded. |
| | **Postconditions:** one Trick base tick advanced; FSW `juno::sch::SCH_API_T<8, 200>::Execute(tSch)` invoked, advancing the scheduler by one minor frame and dispatching whichever apps are due on this 1 ms boundary per `system_design.md` §8.2 (canonical entry point per `conventions.md` §1.4 — supersedes the legacy `sch_lib::Run` name). |
| | **Errors:** none — soft errors logged through failure handler. |
| | **Thread safety:** Single-threaded inside Trick sim-object; not thread-safe across threads. |
| `CaptureBus` | `void CaptureBus() noexcept` |
| | **Preconditions:** `Init` succeeded. |
| | **Postconditions:** every published bus message in this tick appended to capture ring. |
| | **Errors:** none. |
| | **Thread safety:** Single-threaded inside Trick sim-object; not thread-safe across threads. |
| `WriteTruthRow` | `void WriteTruthRow() noexcept` |
| | **Preconditions:** `Init` succeeded; capture ring drained. |
| | **Postconditions:** one CSV/binary row appended to comparison artifact (`SW-REQ-SIM-HARN-009`). |
| | **Errors:** none. |
| | **Thread safety:** Single-threaded inside Trick sim-object; not thread-safe across threads. |
| `FinalizeArtifacts` | `void FinalizeArtifacts() noexcept` |
| | **Preconditions:** `Init` succeeded. |
| | **Postconditions:** all artifact files closed, fsync'd; GPS pty master and slave fds closed (`_iGpsPtyMasterFd`, `_iGpsPtySlaveFd`); SD log path emitted to summary (`SW-REQ-SIM-HARN-007`); telemetry transcript closed (`SW-REQ-SIM-HARN-008`). |
| | **Errors:** none. |
| | **Thread safety:** Single-threaded inside Trick sim-object; not thread-safe across threads. |

All public functions `noexcept`; no constructors / destructors on
`SIM_HARNESS_T` (`conventions.md` §1.3).
