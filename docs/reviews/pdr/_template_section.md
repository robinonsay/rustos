# Juno FT1 PDR — Section {{section_number}}: {{section_title}}

> **How to use this template.** Copy this file to
> `docs/reviews/pdr/sections/S{{section_number}}_<short_name>.md` for the
> section being reviewed. Replace every `{{placeholder}}` with the actual
> value. Leave a placeholder in place only if the field is genuinely not
> applicable; in that case write `N/A` instead of deleting the line.

## 1. Header

| Field | Value |
|-------|-------|
| Section Number | S{{section_number}} |
| Section Title | {{section_title}} |
| Date Convened | {{date}} |
| Chair | {{chair_name}} |
| Software Lead (Presenter) | {{software_lead_name}} |
| Attendees | {{attendee_list}} |

### Documents Under Review

List every artifact presented for this section using its repository path.

- {{doc_path_1}}
- {{doc_path_2}}
- {{doc_path_3}}

## 2. Section Summary

> Software Lead's one-page presentation summary. Capture the same content
> the Lead presented to the room so the record is self-contained.

**Key Decisions Presented**

- {{decision_1}}
- {{decision_2}}

**Key Requirements Covered**

- {{requirement_id_1}} — {{one_line_summary}}
- {{requirement_id_2}} — {{one_line_summary}}

**Key Interfaces Covered**

- {{interface_1}}
- {{interface_2}}

**Risks and Open Issues Flagged by Lead**

- {{risk_1}}

## 3. RID List

Every RID raised during this section. Append rows under the appropriate
source-reviewer subsection. ID format: `PDR-RID-S{{section_number}}-NNN`.

Columns:

| Column | Meaning |
|--------|---------|
| ID | RID identifier (see master log conventions). |
| Severity | Major / Minor / Editorial. |
| Title | Short label, ≤80 characters. |
| Description | Full statement of the discrepancy. |
| Recommended Resolution | What the reviewer suggests be done. |
| Disposition | OPEN until Chair decides; then ACCEPT / ACCEPT-MOD / REJECT / DEFER / CLOSE-NO-ACTION. |
| Owner | Person or role responsible for corrective action. |
| Target | Target close date (YYYY-MM-DD). |
| Status | OPEN / DISPOSED / CLOSED. |

### 3.1 MAE Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|

### 3.2 SSE-R Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|

### 3.3 CE Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|

## 4. RFA List

Advisory items requesting action or investigation. ID format:
`PDR-RFA-S{{section_number}}-NNN`. RFAs do **not** carry a severity field.

### 4.1 MAE RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|

### 4.2 SSE-R RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|

### 4.3 CE RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|

## 5. Disposition Decisions

Chair records per-item decisions here in the order they were rendered.
Use the bullet form below; rationale is mandatory for every disposition
other than a plain ACCEPT of an unmodified recommendation.

- `[{{rid_or_rfa_id}}]: {{DISPOSITION}} — {{rationale_and_modification_notes}}`
- `[{{rid_or_rfa_id}}]: {{DISPOSITION}} — {{rationale_and_modification_notes}}`

## 6. Action Items Created

Each accepted RID or RFA that requires work generates one or more action
items tracked in this table. Action IDs are scoped to this section record
and follow the format `S{{section_number}}-AI-NNN`.

| Action ID | Source RID/RFA | Description | Owner | Target Date | Status |
|-----------|----------------|-------------|-------|-------------|--------|

Status values: OPEN, IN-PROGRESS, DONE, BLOCKED, CANCELLED.

## 7. Section Verdict

The Chair selects exactly one of the three verdicts below by removing the
square brackets from the chosen line and deleting the other two. The
verdict line must be filled in before the section record is considered
final.

- [CHAIR PROCEED] — Section content is acceptable; PDR may proceed to the next section. Open action items will be tracked but do not block progression.
- [CHAIR HOLD] — Section content has unresolved Major RIDs; the section must reconvene after corrective action before the PDR can proceed past this point.
- [BLOCKED] — Review cannot complete due to missing artifacts, missing reviewers, or external dependencies. Reasons must be recorded under "Verdict Notes" below.

**Verdict Notes**

{{chair_verdict_notes}}

**Chair Signature**: {{chair_name}} — {{date}}

## 8. Cross-References

### Documents Reviewed

Repeat the file paths from Section 1 here as clickable repo-relative
links so the record is navigable.

- [{{doc_path_1}}](../../../{{doc_path_1}})
- [{{doc_path_2}}](../../../{{doc_path_2}})

### Master Log

- [PDR RID/RFA Master Log](../rid_rfa_log.md)

### Related Section Records

List any other section records that reference items raised here, or that
this section re-opens (per the master log Cross-Section Re-Open table).

- {{related_section_record_1}}
