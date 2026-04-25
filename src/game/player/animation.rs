use std::time::Duration;
use bevy::prelude::*;

const SOLDIER_MODEL_PATH: &str = "models/Soldier.glb";

#[derive(Resource)]
pub struct SoldierAnimations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}

/// Initializes the soldier animation system
pub fn init_soldier_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Try to load available animations from the Soldier model
    // Most GLB files have animations stored as separate clips
    // We'll try to load multiple animation indices
    let mut animation_clips = vec![];

    // Try to load animations 0-5 (common range for GLB models)
    for i in 0..6 {
        animation_clips.push(
            asset_server.load(GltfAssetLabel::Animation(i).from_asset(SOLDIER_MODEL_PATH)),
        );
    }

    let (graph, node_indices) = AnimationGraph::from_clips(animation_clips);
    let graph_handle = graphs.add(graph);

    // Log how many animations were found
    info!("Loaded {} animations from Soldier model", node_indices.len());

    commands.insert_resource(SoldierAnimations {
        animations: node_indices,
        graph_handle,
    });
}

/// Load the soldier model as a scene
pub fn load_soldier_model(asset_server: &AssetServer) -> Handle<Scene> {
    asset_server.load(GltfAssetLabel::Scene(0).from_asset(SOLDIER_MODEL_PATH))
}

/// Setup animations when the model is loaded
pub fn setup_animations_on_load(
    mut commands: Commands,
    animations: Res<SoldierAnimations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        if !animations.animations.is_empty() {
            info!("Playing first available animation (index 0)");
            transitions
                .play(&mut player, animations.animations[0], Duration::ZERO)
                .repeat();

            commands
                .entity(entity)
                .insert(AnimationGraphHandle(animations.graph_handle.clone()))
                .insert(transitions);
        } else {
            warn!("No animations found in Soldier model!");
        }
    }
}
