//! BGMと効果音を再生
//! BGM: 常時再生
//! SE: メッセージ経由

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

/// ターン終了ボタンが押されたことを伝えるメッセージ
#[derive(Message, Default)]
pub struct NextTurnSfx;

/// 読み込み済みの音声ハンドル
#[derive(Resource)]
struct AudioHandles {
    /// BGMトラック群
    bgm: Vec<Handle<AudioSource>>,
    /// ターン終了の効果音
    next_turn: Handle<AudioSource>,
}

/// 次に再生するBGMトラックの添字
#[derive(Resource, Default)]
struct BgmCursor(usize);

/// 現在鳴っているBGMを指すマーカー
#[derive(Component)]
struct Bgm;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NextTurnSfx>()
            .init_resource::<BgmCursor>()
            .add_systems(Startup, load_audio)
            .add_systems(Update, (ensure_bgm, play_next_turn_sfx));
    }
}

/// 音声ファイルを登録
fn load_audio(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(AudioHandles {
        bgm: vec![
            assets.load("bgm/Efficiency_at_Midday.mp3"),
            assets.load("bgm/Sunday_in_the_Drafting_Room.mp3"),
        ],
        next_turn: assets.load("se/next_turn.mp3"),
    });
}

/// BGMを鳴らしているエンティティが1つも無ければ次のトラックを開始する
fn ensure_bgm(
    mut commands: Commands,
    handles: Option<Res<AudioHandles>>,
    mut cursor: ResMut<BgmCursor>,
    playing: Query<(), With<Bgm>>,
) {
    let Some(handles) = handles else { return };
    if !playing.is_empty() || handles.bgm.is_empty() {
        return;
    }
    let track = handles.bgm[cursor.0 % handles.bgm.len()].clone();
    cursor.0 = cursor.0.wrapping_add(1);
    commands.spawn((
        Bgm,
        AudioPlayer(track),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(0.35),
            ..default()
        },
    ));
}

/// 次のターンのためのSEを鳴らす
fn play_next_turn_sfx(
    mut commands: Commands,
    mut requests: MessageReader<NextTurnSfx>,
    handles: Option<Res<AudioHandles>>,
) {
    let Some(handles) = handles else {
        requests.clear();
        return;
    };
    // 1ターンで複数要求が来ても効果音は1回で十分
    if requests.read().next().is_some() {
        requests.clear();
        commands.spawn((
            AudioPlayer(handles.next_turn.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(0.8),
                ..default()
            },
        ));
    }
}
