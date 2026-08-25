# Design Documents Index — Juno FSW

Design documents follow IEEE 1016. Each module has its own directory under `docs/design/`. Cross-cutting documents (`conventions.md`, `system/system_design.md`) anchor the per-module designs.

## Cross-Cutting Documents

| Document | Path | Purpose |
|----------|------|---------|
| Design Conventions | [conventions.md](conventions.md) | Authoritative cross-module names, idioms, IEEE 1016 structure, vocabulary |
| System Architecture | [system/system_design.md](system/system_design.md) | L1 system design — composition root, bus catalog, scheduler periods, lifecycle |

## FSW Libraries (15)

| Module | Design Doc | Reqs | Status |
|--------|-----------|------|--------|
| gps_lib | [gps/design.md](gps/design.md) | 10 | Approved |
| nmea_lib | [nmea/design.md](nmea/design.md) | 12 | Approved |
| device_lib | [device/design.md](device/design.md) | 7 | Approved |
| kmat_lib | [kmat/index.md](kmat/index.md) (split: index + 04_interface + 05_through_11) | 15 | Approved |
| log_lib | [log/design.md](log/design.md) | 8 | Approved |
| sch_lib | [sch/design.md](sch/design.md) | 10 | Approved |
| time_lib | [time/design.md](time/design.md) | 7 | Approved |
| imu_lib | [imu/design.md](imu/design.md) | 14 | Approved |
| baro_lib | [baro/design.md](baro/design.md) | 10 | Approved |
| lora_lib | [lora/design.md](lora/design.md) | 12 | Approved |
| sd_lib | [sd/design.md](sd/design.md) | 12 | Approved |
| nav_lib | [nav/design.md](nav/design.md) | 17 | Approved |
| afm_lib | [afm/design.md](afm/design.md) | 11 | Approved |
| telem_lib | [telem/design.md](telem/design.md) | 12 | Approved |
| mlog_lib | [mlog/design.md](mlog/design.md) | 14 | Approved |

## FSW Applications (8)

| Module | Design Doc | Reqs | Status |
|--------|-----------|------|--------|
| gps_app | [gps_app/design.md](gps_app/design.md) | 10 | Approved |
| imu_app | [imu_app/design.md](imu_app/design.md) | 10 | Approved |
| baro_app | [baro_app/design.md](baro_app/design.md) | 10 | Approved |
| nav_app | [nav_app/design.md](nav_app/design.md) | 13 | Approved |
| afm_app | [afm_app/design.md](afm_app/design.md) | 10 | Approved |
| telem_app | [telem_app/design.md](telem_app/design.md) | 11 | Approved |
| mlog_app | [mlog_app/design.md](mlog_app/design.md) | 12 | Approved |
| sys_app | [sys_app/design.md](sys_app/design.md) | 12 | Approved |

## Simulation Modules (4)

| Module | Design Doc | Reqs | Status |
|--------|-----------|------|--------|
| sim_dynamics | [sim_dynamics/design.md](sim_dynamics/design.md) | 14 | Approved |
| sim_sensors | [sim_sensors/design.md](sim_sensors/design.md) | 14 | Approved |
| sim_scenario | [sim_scenario/design.md](sim_scenario/design.md) | 12 | Approved |
| sim_harness | [sim_harness/design.md](sim_harness/design.md) (split: design + interfaces.md) | 10 | Approved |

## Coverage Summary

- 15 libraries + 8 applications + 4 simulation modules = **27 module designs**
- All 371 `SW-REQ-*` requirement IDs covered by `<!-- @{"design": [...]} -->` tags
- `python3 tools/traceability.py` exits 0
- Every file ≤500 lines (per `constraints.md`); kmat and sim_harness are split per the index-file rule

## Design Document Structure (IEEE 1016)

Each per-module design follows the 11-section structure documented in [conventions.md](conventions.md) §7:

1. **Purpose and Scope** — what capability is being designed
2. **Definitions and Abbreviations** — terms used in the document
3. **System Overview** — MVC layer mapping for this module
4. **Interface Definitions** — all public APIs with contracts
5. **State Machines** — state diagrams for stateful components
6. **Data Flow** — message types and directions on the software bus
7. **Sequence Diagrams** — interaction sequences between components
8. **Timing and Scheduling** — TDM period assignments, deadlines
9. **Error Handling Strategy** — how errors propagate and are reported
10. **Memory Ownership** — who allocates what, lifetimes
11. **Traceability** — which requirements each section addresses

## Traceability Tags

Tag each major section with the requirements it addresses (per [conventions.md](conventions.md) §8):

```markdown
<!-- @{"design": ["SW-REQ-GPS-001", "SW-REQ-GPS-002"]} -->
### 4.1 UART Interface Contract
```

Per-section tags are authoritative; the §11 traceability table in each module's design is descriptive consolidation.
