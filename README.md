# LoL Champion Simulation Engine

Rust 기반 리그 오브 레전드(League of Legends) 1:1 챔피언 교전 시뮬레이션 엔진.  
SimulationCraft(simc)의 핵심 이벤트 루프 패턴을 차용하여 LoL의 고유 전투 메커니즘을 시뮬레이션하고 상세 통계 및 인터랙티브 타임라인 리포트를 제공합니다.

---

## 🚀 주요 특징 (Key Features)

1. **이벤트 기반 시뮬레이션 (Event-Driven)**: Timing Wheel 기반의 이벤트 매니저 루프를 통해 모든 액션, 버프 틱, 평타 쿨타임 등을 시뮬레이션합니다.
2. **데이터 구동식 설계 (Data-Driven)**: 챔피언 데이터, 아이템 스펙 및 성장 능력치 등을 외부 JSON 파일로 관리하여 코드 수정 없이 밸런스 패치에 대응합니다.
3. **스탯 3계층 아키텍처 (Three-Layer Stats)**: `Base(기본) → Initial(아이템+룬) → Current(동적 버프 반영)`로 나뉜 유연한 스탯 계산 파이프라인.
4. **Action Priority List (APL)**: 각 챔피언의 스킬 우선순위 및 사용 조건을 표현하는 파서와 인터프리터를 내장하여 스마트한 AI 교전 시나리오를 구성할 수 있습니다.
5. **다크 모드 인터랙티브 리포트 (HTML Output)**: 
   - OP.GG CDN을 통한 챔피언/스킬/아이템 아이콘 자동 바인딩.
   - 자원 변화 추이(체력/마나/기력 등)를 한눈에 볼 수 있는 동적 꺾은선 그래프 및 간트 차트 지원.
   - **스킬 시전 시 자원 소모량(마나/기력/분노 등) 실시간 노출** 및 피들스틱 기반 Target Dummy 5초 완전 회복 기믹 시각화.

---

## 🧱 프로젝트 아키텍처 (Architecture)

프로젝트는 다음과 같은 독립적인 Crate 단위로 설계되었습니다.

```text
crates/
├── lol-core/       # 핵심 시뮬레이션 엔진 (이벤트 루프, 스탯 시스템, 데미지 계산, 룬 매니저)
├── lol-data/       # 외부 JSON 게임 데이터(아이템, 룬, 스탯) 로더
├── lol-champions/  # 챔피언별 스킬셋 및 상태 머신 정의 (Garen, Darius, Zed, Jinx, Ahri, Dummy)
├── lol-apl/        # 조건문 파서 및 우선순위 행동(Action Priority List) 실행기
└── lol-report/     # 시뮬레이션 통계 수집, 텍스트 포맷터 및 HTML 인터랙티브 템플릿 생성
```

### 🧬 지원 메커니즘 및 룬/아이템
- **챔피언**: 가렌, 다리우스, 제드(기력 자원 사용), 징크스(Pow-Pow/Fishbones 폼 스위칭 기믹 및 공속/사거리 반영), 아리(매혹 및 데미지 증폭), Target Dummy (피들스틱 18레벨 고정 및 5초 주기 HP 100% 회복 기믹 탑재)
- **핵심 룬**: 집공(Press the Attack - 취약 디버프 적용), 감전(Electrocute - 적중 스택 기반 적응형 피해)
- **하위 룬**: 착취의 손아귀, 승전보, 전설: 민첩함, 최후의 저항, 뼈 방패, 과잉성장
- **아이템 Passives**: 무한의 대검(치명타 피해 증폭), 필멸자의 운명(치유 감소 40% 디버프), 유령 무희(평타 적중 시 이동 속도 및 4스택 추가 공속 버프)

---

## 🛠️ CLI 사용법 (CLI Usage)

본 프로젝트는 `clap` 기반의 터미널 인터페이스를 제공합니다.

### 1. 시뮬레이션 실행 및 결과 콘솔 출력
```bash
cargo run -- simulate -a Jinx -b Darius
```
- `-a`, `--champion-a <NAME>`: 조종할 챔피언 이름 (첫 글자 대문자)
- `-b`, `--champion-b <NAME>`: 피격자 챔피언 이름 (지정하지 않으면 기본값 `Dummy`로 설정)

### 2. 마나 조건식을 포함한 APL 파일 주입 실행
징크스 Q스킬 폼 스위칭 시 마나가 부족하면 미니건으로 롤백해주는 마나 감지 조건이 포함된 APL을 전달하여 시뮬레이션을 수행할 수 있습니다.
```bash
cargo run -- simulate -a Jinx -b Dummy --apl garen_test.txt --html-out report.html
```

### 3. 통계용 반복 횟수 지정 (Iterations)
치명타, 난수 기반 발동 효과 등을 포함하여 평균적인 DPS 및 데미지 기여도를 측정할 때 사용합니다. (기본값: 100회)
```bash
cargo run -- simulate -a Jinx -b Dummy --iterations 1000
```

---

## 🧪 테스트 및 품질 검증 (Testing & Verification)

### 1. 전체 워크스페이스 유닛/통합 테스트
```bash
cargo test --workspace
```
스탯 성장 공식, 관통력 계산, 룬 발동 조건식, 챔피언 고유 스태틱 및 회복 로직에 대한 80개 이상의 테스트가 포함되어 있습니다.

### 2. Playwright 시각화/오류 탐지 검증
생성된 HTML 리포트 내에 자바스크립트 런타임 오류가 없으며 화면 컴포넌트가 깨지지 않고 로드되는지 가상 브라우저 상에서 스크린샷과 콘솔 로그를 검사합니다.
```bash
# Playwright 스크립트 실행 (미리 report.html이 생성되어 있어야 합니다)
node playwright-test/verify.js
```

---
*Developed under Advanced Agentic Coding practices.*
