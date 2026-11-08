## ADDED Requirements

### Requirement: AS-1 immutable exact audio segments

The provider SHALL decode only supplied verified immutable bytes and a matching fresh request. It SHALL return only actual decoded finite samples in the selected half-open source segment and preserve the complete encoded source.

#### Scenario: Source segment is decoded and sought
- **WHEN** a valid source segment is opened and sought within its range
- **THEN** the returned packets SHALL begin at the exact requested source frame after codec preroll and end before the exclusive segment boundary
- **AND** the decoder position SHALL not be described as an audible device clock

### Requirement: AS-2 explicit bounded failure

The provider SHALL retain unknown duration and damaged-packet facts, enforce independent channel/rate/packet/duration limits, and cooperate with caller cancellation around backend operations.

#### Scenario: Request is stale or capacity is exceeded
- **WHEN** a foreign request, cancelled operation, invalid segment or excessive decoded packet is encountered
- **THEN** the provider SHALL return an explicit error without mutating or silently truncating original content
