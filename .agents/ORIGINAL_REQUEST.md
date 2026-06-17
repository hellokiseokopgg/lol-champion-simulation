# Original User Request

## Initial Request — 2026-06-17T07:01:30Z

# Teamwork Project Prompt — Draft

> Status: Step 6 — Reviewing Draft
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

LoL 챔피언 시뮬레이션 엔진에 감전(Electrocute), 집중 공격(Press the Attack) 등 누적 타격/스택 기반의 적응형 추가 데미지 룬과 관련 메커니즘을 추가합니다.

Working directory: /Users/kskim/Projects/lol-champion-simulation
Integrity mode: development

## Requirements

### R1. Implement Electrocute and Press the Attack
- Add the `Electrocute` and `Press the Attack` runes to the engine's rune manager.
- Implement logic to track distinct attacks/abilities within the specified time window.
- Apply the appropriate adaptive damage (Electrocute) or exposure debuff + burst damage (Press the Attack) when the conditions are met.

### R2. Update Event and Damage Pipelines
- Ensure that the damage pipeline can handle adaptive bonus damage and percentile damage amplification (for Press the Attack's exposure effect).
- Emit appropriate timeline events when the runes trigger so they appear correctly on the HTML Gantt chart.

## Acceptance Criteria

### Simulation Correctness
- [ ] Running a CLI simulation with Garen/Darius using `Electrocute` correctly shows "Electrocute" damage in the DPS breakdown after 3 separate hits.
- [ ] Running a CLI simulation with `Press the Attack` correctly applies the burst damage and amplifies subsequent damage by the correct percentage.
- [ ] Both runes must properly track cooldowns (e.g., Electrocute cannot trigger twice within its cooldown window).

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*
