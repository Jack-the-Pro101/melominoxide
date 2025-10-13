// TODO: rename this probably
pub enum Dimension {
    Overworld,
    Nether,
    End,
    Minecraft,
    Disc,
}

pub fn song_to_dimension(filename: &str) -> Dimension {
    match filename {
        // Overworld tracks
        "aerie.ogg"
        | "aria_math.ogg"
        | "ancestry.ogg"
        | "a_familiar_room.ogg"
        | "an_ordinary_day.ogg"
        | "below_and_above.ogg"
        | "biome_fest.ogg"
        | "blind_spots.ogg"
        | "broken_clocks.ogg"
        | "bromeliad.ogg"
        | "clark.ogg"
        | "comforting_memories.ogg"
        | "crescent_dunes.ogg"
        | "danny.ogg"
        | "deeper.ogg"
        | "dreiton.ogg"
        | "dry_hands.ogg"
        | "echo_in_the_wind.ogg"
        | "eld_unknown.ogg"
        | "endless.ogg"
        | "featherfall.ogg"
        | "firebugs.ogg"
        | "fireflies.ogg"
        | "floating_dream.ogg"
        | "haggstrom.ogg"
        | "haunt_muskie.ogg"
        | "infinite_amethyst.ogg"
        | "key.ogg"
        | "komorebi.ogg"
        | "labyrinthine.ogg"
        | "left_to_bloom.ogg"
        | "lilypad.ogg"
        | "living_mice.ogg"
        | "mice_on_venus.ogg"
        | "minecraft.ogg"
        | "one_more_day.ogg"
        | "os_piano.ogg"
        | "oxygene.ogg"
        | "pokopoko.ogg"
        | "puzzlebox.ogg"
        | "stand_tall.ogg"
        | "subwoofer_lullaby.ogg"
        | "sweden.ogg"
        | "taswell.ogg"
        | "watcher.ogg"
        | "wending.ogg"
        | "wet_hands.ogg"
        | "yakusoku.ogg"
        | "axolotl.ogg"
        | "dragon_fish.ogg"
        | "shuniji.ogg" => Dimension::Overworld,

        // Nether tracks
        "ballad_of_the_cats.ogg"
        | "chrysopoeia.ogg"
        | "concrete_halls.ogg"
        | "dead_voxel.ogg"
        | "rubedo.ogg"
        | "so_below.ogg"
        | "warmth.ogg" => Dimension::Nether,

        // End tracks
        "boss.ogg" | "the_end.ogg" | "alpha.ogg" => Dimension::End,

        // Disc tracks
        "11.ogg"
        | "13.ogg"
        | "5.ogg"
        | "blocks.ogg"
        | "cat.ogg"
        | "chirp.ogg"
        | "far.ogg"
        | "mall.ogg"
        | "mellohi.ogg"
        | "stal.ogg"
        | "strad.ogg"
        | "wait.ogg"
        | "precipice.ogg"
        | "relic.ogg"
        | "creator_music_box.ogg"
        | "creator.ogg"
        | "pigstep.ogg"
        | "otherside.ogg"
        | "ward.ogg"
        | "tears.ogg"
        | "lava_chicken.ogg" => Dimension::Disc,

        _ => Dimension::Minecraft,
    }
}
