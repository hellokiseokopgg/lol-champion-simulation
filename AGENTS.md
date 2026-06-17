# AGENTS.md — LoL Champion Simulation

## Project Overview

Rust 기반 리그 오브 레전드 1:1 챔피언 교전 시뮬레이션 엔진.
SimulationCraft(simc)의 핵심 아키텍처 패턴을 차용하여 LoL의 전투 메커니즘을 시뮬레이션합니다.

## Architecture

### Crate 구조

```
crates/
├── lol-core/       # 핵심 엔진 (이벤트 루프, 스탯, 데미지 파이프라인, 버프)
├── lol-data/       # 게임 데이터 로딩 (JSON → Rust 구조체)
├── lol-champions/  # 챔피언별 모듈 (각 챔피언 1파일)
├── lol-apl/        # Action Priority List 파서/실행기
└── lol-report/     # 통계 수집 및 리포팅
```

### 핵심 설계 원칙

1. **Event-Driven**: 모든 게임 액션은 Timing Wheel 이벤트로 처리
2. **Data-Driven**: 챔피언/아이템/룬 수치는 JSON 파일로 외부화
3. **Module-per-Champion**: 챔피언 추가 시 엔진 코드 변경 불필요
4. **Three-Layer Stats**: `base → initial → current` 3계층 스탯 시스템

### 의존 관계

```
lol-core (독립)
lol-data → lol-core
lol-apl → lol-core
lol-champions → lol-core, lol-data
lol-report → lol-core
main.rs → 모든 crate
```

## Coding Conventions

### Rust Style
- `rustfmt` 기본 설정 사용
- `clippy` 경고 0개 유지
- 모든 공개 타입/함수에 `///` doc comment 필수
- `unwrap()` 금지 — `Result`/`Option` 적절히 처리
- 에러 타입은 `thiserror`로 정의

### Naming
- 타입: `PascalCase` (예: `StatBlock`, `DamagePipeline`)
- 함수/변수: `snake_case` (예: `calculate_damage`, `attack_damage`)
- 상수: `SCREAMING_SNAKE_CASE` (예: `MAX_ATTACK_SPEED`)
- 모듈: `snake_case` (예: `champion_data.rs`)
- 챔피언 모듈: 챔피언 영문명 소문자 (예: `garen.rs`, `jinx.rs`)

### 파일 구조
- 각 `.rs` 파일은 하나의 관심사만 담당
- `lib.rs`는 모듈 선언과 re-export만 포함
- 테스트는 같은 파일의 `#[cfg(test)] mod tests` 블록에 작성

## Data Format

### 챔피언 데이터 (JSON)
```
data/champions/{champion_id}.json
```
- 기본 스탯, 성장치, 스킬 데이터 포함
- 스킬 계수는 레벨별 배열로 관리
- 밸런스 패치 시 이 파일만 수정

### 아이템/룬 데이터 (JSON)
```
data/items.json
data/runes.json
```
- 아이템 `id`는 라이엇 공식 아이템 ID 체계 (예: `"3071"`)를 따릅니다.
- **메타 데이터 참조 API**: `https://ddragon.leagueoflegends.com/cdn/16.12.1/data/en_US/item.json`
- **아이템 아이콘(CDN) 이미지 URL 패턴**:
  - `https://opgg-static.akamaized.net/meta/images/lol/16.12.1/item/{ITEM_ID}.png?image=q_auto:good,f_webp,w_64,h_64&v=1612`
  - HTML 리포트 등에서 아이템 이미지를 렌더링할 때 위 OP.GG CDN 패턴을 유추하여 사용합니다.

## Key Patterns

### 챔피언 모듈 작성법
1. `ChampionModule` trait 구현 (팩토리)
2. 내부 struct에 챔피언 고유 상태 (패시브 스택 등) 포함
3. `Ability` trait으로 Q/W/E/R/AA 구현
4. `register()` 함수로 `ChampionRegistry`에 등록

### 데미지 계산 흐름
```
Ability::calculate_damage()
  → DamagePipeline::process()
    → Stage 1: Raw damage (base + ratios)
    → Stage 2: Flat resistance reduction
    → Stage 3: % resistance reduction
    → Stage 4: % penetration
    → Stage 5: Flat penetration
    → Stage 6: Mitigation (100 / (100 + effective_resist))
    → Stage 7: Shield absorption
    → Stage 8: Final damage application
```

### 이벤트 루프
```
EventManager::run()
  → pop next event (lowest time)
  → advance sim_time
  → event.execute(sim_context)
  → (event may schedule new events)
  → repeat until no events or max_time
```

## Testing

- **UI 및 리포트 시뮬레이션 검증**: 특별한 명시가 없는 한 항상 **Dummy**를 타겟(`-b Dummy`)으로 사용하여 시뮬레이션 및 리포트를 생성/검증할 것.
- `cargo test --workspace` — 전체 테스트
- 각 공식(데미지, 스탯 성장, 관통)에 대한 유닛 테스트 필수
- 챔피언 모듈은 "known damage" 시나리오 테스트 포함
- 통합 테스트: `tests/integration/` 디렉토리

## Dependencies

| Crate | Purpose |
|-------|---------|
| serde, serde_json | 데이터 직렬화 |
| rand | RNG (크리티컬, 확률) |
| rayon | 멀티스레드 병렬화 |
| clap | CLI 파싱 |
| thiserror | 에러 타입 |
| tracing | 로깅 |
| ordered-float | 이벤트 큐 정렬 |

## CLI Usage

이 시뮬레이션 프로젝트는 `clap` 기반의 터미널 인터페이스를 제공합니다. 메인 명령어는 `simulate`입니다.

### 시뮬레이션 실행 (텍스트 & 간트 차트 출력)
```bash
cargo run -- simulate -a Garen -b Darius
```
- `-a`, `--champion-a <NAME>`: 조종할 챔피언 이름 (예: Garen, Jinx 등). 대소문자를 구분할 수 있으므로 가급적 첫 글자를 대문자로 입력하세요.
- `-b`, `--champion-b <NAME>`: 샌드백용 혹은 적 챔피언 이름 (예: Darius).

### 타임라인 웹 리포트 출력 (HTML)
```bash
cargo run -- simulate -a Garen -b Darius --html-out report.html
```
- `--html-out <FILE_PATH>`: 텍스트 및 간트 차트 외에 OP.GG 아이콘이 연동된 다크 테마 기반의 "웹 브라우저용 인터랙티브 타임라인 리포트" 파일을 생성합니다. 생성된 `report.html` 파일을 브라우저로 열면 됩니다.

### 반복 횟수 지정 (통계용)
```bash
cargo run -- simulate -a Garen -b Darius --iterations 1000
```
- `-i`, `--iterations <NUM>`: 시뮬레이션을 몇 회 반복할지 설정합니다. 치명타, 회피, 발동 확률 등의 난수 효과에 대한 정밀한 평균 DPS를 구하기 위해 사용됩니다. (기본값: 100)

## References

- [GSE Builder Macro Reference](https://tools.wowlazymacros.com/gse-builder?_gl=1*14t8b2v*_ga*MTA0NjQxNTQ1Ni4xNzc2OTE5OTEz*_ga_CZH3DPC08F*czE3ODE2MTA2NTQkbzQkZzEkdDE3ODE2MTIyNTkkajU2JGwwJGgw&name=SCG_FRO_AOE&class=deathknight&specid=251&author=Detnit%40Echo+Isles#share=!GSE3!1ZQ7b9NQFIDTtEQMSJR2oRVDJSYWEjdN+pgItuM4zQOaUBAM1Y190li5sV3f60phQEFsSIiZCakvEGJDIFgQDJVAYkCCseJX0KFiAKd5+DaprUgVQmz2tb/zsM/5NiQT1bGB1C15GVkaKmEgjWQWKZZBGhkRQw10yhu2ToNyAdZs0BUgG4sFXlpJLuVXEnlxS74JqJqwLUQaqWWwiGbo5P7mNVkv2RqmblQpodDms0c7IjEB44nXBwKtm9C6S8k6BWsd4URUKDqnydbbgqwY+mRg9PNDwdY1emWHQdphPj73Z4ef7nmy7766bKzFLoEJiLbZ4OgXL3YyEHpzsS/zEfq2teuZ+ZPknzn0+0eHZVrqpB4/d5dN7UZv02dfPW7THWL40j02iy+99mC5hw4eILYMX/p09IVn1x+mXHTumH8VHHvi0/UIH/LNfGp270jdE+/5gasOjr9tsdtHytIo1K5y0V708Pzw/aSJUR2sbea4xcx4TSVDiRlUAiwJUEY2posZROgNU0UU1Nx0ZDoeiXMxLs5FYzOpLFAkIIpeplKATazp1VsXKpSaZCEcVjWiGJZ6eXU1vEqgGR5UqYiws7nOLidsWjGs6wLQZneiUjGmZOLsY1oqiO11nThzsJRFuo1wq2QHdE73U3wFlCqxa4vr3IKCUJmbny13ih0qinrZsBTgjZrpFFbSsEbr+xnBGS9ddTShAdnsiIRxi2uRRrJggiIL53/xxTw/GRj6vivkUA1YuaQ714XiiTwz6BS4nunKZWBHuZ7pyuVY1t8zjFz8Mnt4piuX43c91t8x45l+w/Vopr/nfs+MfMsPXHdw7E4vzYcG/t7utp/AM8/YdIJGmiOk/vSYl65LAxvdOBVPXzn2+U+NE5nnIv/eONMliKrz6tzfNw5jmcOPmubzmYzIF+V87g8=)
