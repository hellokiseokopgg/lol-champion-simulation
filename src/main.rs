use clap::{Parser, Subcommand};
use lol_champions::ChampionRegistry;
use lol_core::champion::ChampionConfig;
use tracing::{info, Level};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a 1v1 champion combat simulation
    Simulate {
        /// First champion name (e.g., Garen)
        #[arg(short = 'a', long)]
        champion_a: String,

        /// Second champion name (e.g., Darius)
        #[arg(short = 'b', long)]
        champion_b: Option<String>,

        /// Number of iterations to run
        #[arg(short, long, default_value_t = 100)]
        iterations: u32,

        /// Output HTML report path (e.g., report.html)
        #[arg(long)]
        html_out: Option<String>,

        /// Language for report output (e.g., ko, en)
        #[arg(long, default_value = "ko")]
        lang: String,

        /// OP.GG Match URL to fetch item build automatically
        #[arg(long)]
        opgg: Option<String>,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Simulate { champion_a, champion_b, iterations, html_out, lang, opgg } => {
            let champion_b = champion_b.clone().unwrap_or_else(|| "Dummy".to_string());
            info!("Initializing LoL Champion Simulation Engine...");
            info!("Matchup: {} vs {}", champion_a, champion_b);
            info!("Iterations: {}", iterations);

            let registry = ChampionRegistry::new();
            
            let champ_a_module = registry.get(champion_a).unwrap_or_else(|| {
                panic!("Champion {} not found in registry", champion_a);
            });
            let champ_b_module = registry.get(&champion_b).unwrap_or_else(|| {
                panic!("Champion {} not found in registry", champion_b);
            });

            // Load JSON data for champions
            let loader = lol_data::loader::DataLoader::new("data");
            let data_a = loader.load_champion(&champion_a.to_lowercase()).unwrap();
            
            let data_b = if champion_b.eq_ignore_ascii_case("dummy") {
                // Return mock data for Dummy
                lol_data::champion_data::ChampionData {
                    id: "Dummy".to_string(),
                    name: "Target Dummy".to_string(),
                    base_stats: lol_data::champion_data::BaseStats {
                        hp: 1000000.0,
                        mp: 0.0,
                        hp_regen: 0.0,
                        mp_regen: 0.0,
                        armor: 100.0,
                        magic_resist: 100.0,
                        attack_damage: 0.0,
                        attack_speed: 0.0,
                        attack_range: 0.0,
                        move_speed: 0.0,
                    },
                    growth_stats: lol_data::champion_data::GrowthStats {
                        hp: 0.0,
                        mp: 0.0,
                        hp_regen: 0.0,
                        mp_regen: 0.0,
                        armor: 0.0,
                        magic_resist: 0.0,
                        attack_damage: 0.0,
                        attack_speed: 0.0,
                    },
                    skills: Vec::new(),
                }
            } else {
                loader.load_champion(&champion_b.to_lowercase()).unwrap()
            };



            // Create instances
            let base_stats_a = lol_core::stats::StatBlock {
                health: data_a.base_stats.hp as f64,
                health_regen: data_a.base_stats.hp_regen as f64,
                mana: data_a.base_stats.mp as f64,
                mana_regen: data_a.base_stats.mp_regen as f64,
                attack_damage: data_a.base_stats.attack_damage as f64,
                ability_power: 0.0,
                armor: data_a.base_stats.armor as f64,
                magic_resist: data_a.base_stats.magic_resist as f64,
                attack_speed: data_a.base_stats.attack_speed as f64,
                attack_speed_ratio: Some(data_a.base_stats.attack_speed as f64),
                movement_speed: data_a.base_stats.move_speed as f64,
                ..Default::default()
            };
            let all_items = loader.load_all_items().unwrap_or_default();
            let mut item_build_a = lol_core::item::ItemBuild::new();

            // Build setup
            // (We will use timeline events instead of adding them at start)

            let growth_stats_a = lol_core::stats::StatBlock {
                health: data_a.growth_stats.hp as f64,
                health_regen: data_a.growth_stats.hp_regen as f64,
                mana: data_a.growth_stats.mp as f64,
                mana_regen: data_a.growth_stats.mp_regen as f64,
                attack_damage: data_a.growth_stats.attack_damage as f64,
                armor: data_a.growth_stats.armor as f64,
                magic_resist: data_a.growth_stats.magic_resist as f64,
                attack_speed: data_a.growth_stats.attack_speed as f64,
                ..Default::default()
            };

            let growth_stats_b = lol_core::stats::StatBlock {
                health: data_b.growth_stats.hp as f64,
                health_regen: data_b.growth_stats.hp_regen as f64,
                mana: data_b.growth_stats.mp as f64,
                mana_regen: data_b.growth_stats.mp_regen as f64,
                attack_damage: data_b.growth_stats.attack_damage as f64,
                armor: data_b.growth_stats.armor as f64,
                magic_resist: data_b.growth_stats.magic_resist as f64,
                attack_speed: data_b.growth_stats.attack_speed as f64,
                ..Default::default()
            };

            let config_a = lol_core::champion::ChampionConfig {
                level: 1, // Start at level 1 for timeline sim!
                item_build: item_build_a,
                rune_page: lol_core::rune::RunePage::default(),
                base_stats: base_stats_a,
                growth_stats: growth_stats_a,
            };

            let base_stats_b = lol_core::stats::StatBlock {
                health: data_b.base_stats.hp as f64,
                health_regen: data_b.base_stats.hp_regen as f64,
                mana: data_b.base_stats.mp as f64,
                mana_regen: data_b.base_stats.mp_regen as f64,
                attack_damage: data_b.base_stats.attack_damage as f64,
                ability_power: 0.0,
                armor: data_b.base_stats.armor as f64,
                magic_resist: data_b.base_stats.magic_resist as f64,
                attack_speed: data_b.base_stats.attack_speed as f64,
                attack_speed_ratio: Some(data_b.base_stats.attack_speed as f64),
                movement_speed: data_b.base_stats.move_speed as f64,
                ..Default::default()
            };
            let config_b = ChampionConfig {
                level: 1,
                item_build: lol_core::item::ItemBuild::new(),
                rune_page: lol_core::rune::RunePage::default(),
                base_stats: base_stats_b,
                growth_stats: growth_stats_b,
            };

            let mut sim = lol_core::sim::GameSimulation::new(lol_core::sim::SimConfig {
                max_duration: 1605.0, // 26 minutes 45 seconds = 1605 seconds
            });

            let items_a: Vec<(String, String)> = config_a.item_build.items.iter().map(|i| (i.id.clone(), i.name.clone())).collect();
            let items_b: Vec<(String, String)> = config_b.item_build.items.iter().map(|i| (i.id.clone(), i.name.clone())).collect();

            let inst_a = std::rc::Rc::new(std::cell::RefCell::new(champ_a_module.create_instance(config_a)));
            let inst_b = std::rc::Rc::new(std::cell::RefCell::new(champ_b_module.create_instance(config_b)));

            let id_a = lol_core::types::ChampionId(champion_a.clone());
            let id_b = lol_core::types::ChampionId(champion_b.clone());

            sim.add_actor(id_a.clone(), inst_a);
            sim.add_actor(id_b.clone(), inst_b);

            // For now, we mock the damage directly to test reporting
            let collector = std::rc::Rc::new(std::cell::RefCell::new(lol_report::collector::DataCollector::new()));
            {
                let mut col = collector.borrow_mut();
                col.champion_items.insert(id_a.clone(), items_a);
                col.champion_items.insert(id_b.clone(), items_b);
            }
            
            let garen_apl_script = "
actions+=/R,if=target.health.pct<30
actions+=/AutoAttack,if=buff.Judgment.down
actions+=/Q,if=buff.Judgment.down
actions+=/E
actions+=/W
";
            let darius_apl_script = "
actions+=/R,if=target.buff.Hemorrhage.stack>4
actions+=/Q,if=cooldown.AutoAttack.ready&buff.Crippling Strike.down
actions+=/W
actions+=/AutoAttack
";
            let apl_script = if champion_a.eq_ignore_ascii_case("darius") {
                darius_apl_script
            } else {
                garen_apl_script
            };

            let garen_apl = lol_apl::parser::ActionPriorityList::parse(apl_script).unwrap();

            let tick_event = lol_apl::executor::ActorTickEvent {
                actor: id_a.clone(),
                target: id_b.clone(),
                apl: garen_apl,
            };

            // Start the sequence at time 0.0
            sim.event_manager_mut().schedule(
                lol_core::types::SimTime::new(0.0),
                Box::new(tick_event),
            );

            // Schedule Level Up events (level 1 to 18 over 1600 seconds ~ every 94 seconds)
            for lvl in 2..=18 {
                sim.event_manager_mut().schedule(
                    lol_core::types::SimTime::new((lvl - 1) as f64 * 94.0),
                    Box::new(lol_core::event::LevelUpEvent {
                        target: id_a.clone(),
                    }),
                );
            }

            // Schedule Item Acquisition Events based on the OP.GG screenshot
            // 11 min (660s): Stridebreaker (6631)
            if let Some(stridebreaker) = all_items.iter().find(|i| i.id == "6631") {
                sim.event_manager_mut().schedule(
                    lol_core::types::SimTime::new(660.0),
                    Box::new(lol_core::event::ItemAcquisitionEvent {
                        target: id_a.clone(),
                        item_id: stridebreaker.id.clone(),
                        item_name: stridebreaker.name.clone(),
                        item_stats: stridebreaker.clone().into_item().stats,
                    }),
                );
            }

            // 19 min (1140s): Mortal Reminder (3033)
            if let Some(mortal) = all_items.iter().find(|i| i.id == "3033") {
                sim.event_manager_mut().schedule(
                    lol_core::types::SimTime::new(1140.0),
                    Box::new(lol_core::event::ItemAcquisitionEvent {
                        target: id_a.clone(),
                        item_id: mortal.id.clone(),
                        item_name: mortal.name.clone(),
                        item_stats: mortal.clone().into_item().stats,
                    }),
                );
            }


            sim.run(Some(collector.clone()));

            let max_time = lol_core::types::SimTime::new(60.0);
            let stats = lol_report::statistics::Statistics::calculate(&collector.borrow(), max_time);
            
            let translator = lol_report::i18n::Translator::new(&lang);
            
            let report = lol_report::formatter::Formatter::format_text(&stats, &collector.borrow());
            let gantt = lol_report::formatter::Formatter::format_gantt(&collector.borrow(), &translator);

            println!("\n{}", report);
            println!("\n{}", gantt);

            if let Some(path) = html_out {
                let html = lol_report::formatter::Formatter::format_html(&collector.borrow(), apl_script, &translator);
                std::fs::write(path, html).expect("Failed to write HTML report");
                info!("Saved HTML report to {}", path);
            }
            
            info!("Simulation complete.");
        }
    }
}
