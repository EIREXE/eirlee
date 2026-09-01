use std::io;

use bevy::{
    asset::{
        AssetLoader, AssetPath, AsyncReadExt, AsyncWriteExt, LoadContext,
        io::{Reader, Writer},
        processor::LoadTransformAndSave,
        saver::{AssetSaver, SavedAsset},
        transformer::{AssetTransformer, TransformedAsset},
    },
    prelude::*,
};

use crate::scripting::{FighterAttackScript, move_compiler};

#[derive(Default, TypePath)]
struct FighterAttackScriptSourceLoader;

#[derive(Asset, TypePath)]
struct FighterAttackScriptContents(String);

impl AssetLoader for FighterAttackScriptSourceLoader {
    type Asset = FighterAttackScriptContents;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut contents = String::default();
        reader.read_to_string(&mut contents).await?;
        Ok(FighterAttackScriptContents(contents))
    }

    fn extensions(&self) -> &[&str] {
        &["attack.rhai"]
    }
}

#[derive(Default, TypePath)]
struct FighterAttackScriptTransformer;

impl AssetTransformer for FighterAttackScriptTransformer {
    type AssetInput = FighterAttackScriptContents;
    type AssetOutput = FighterAttackScript;
    type Settings = ();
    type Error = String;

    async fn transform<'a>(
        &'a self,
        source: TransformedAsset<Self::AssetInput>,
        _: &'a Self::Settings,
    ) -> Result<TransformedAsset<Self::AssetOutput>, Self::Error> {
        let output = move_compiler::compile_script(&source.0)?;
        Ok(source.replace_asset(output))
    }
}

#[derive(Default, TypePath)]
struct FighterAttackScriptLoader;

impl AssetLoader for FighterAttackScriptLoader {
    type Asset = FighterAttackScript;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        bincode::deserialize(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn extensions(&self) -> &[&str] {
        &["attack.rhai"]
    }
}

#[derive(Default, TypePath)]
struct FighterAttackScriptSaver;

impl AssetSaver for FighterAttackScriptSaver {
    type Asset = FighterAttackScript;
    type Settings = ();
    type OutputLoader = FighterAttackScriptLoader;
    type Error = io::Error;

    async fn save(
        &self,
        writer: &mut Writer,
        asset: SavedAsset<'_, '_, Self::Asset>,
        _: &Self::Settings,
        _: AssetPath<'_>,
    ) -> Result<(), Self::Error> {
        let bytes = bincode::serialize(&*asset)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        writer.write_all(&bytes).await
    }
}

type FighterAttackScriptProcessor = LoadTransformAndSave<
    FighterAttackScriptSourceLoader,
    FighterAttackScriptTransformer,
    FighterAttackScriptSaver,
>;

pub struct FighterScriptImportPlugin;

impl Plugin for FighterScriptImportPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FighterAttackScript>()
            .register_asset_loader(FighterAttackScriptSourceLoader)
            .register_asset_loader(FighterAttackScriptLoader)
            .register_asset_processor(FighterAttackScriptProcessor::new(
                FighterAttackScriptTransformer,
                FighterAttackScriptSaver,
            ))
            .set_default_asset_processor::<FighterAttackScriptProcessor>("attack.rhai");
    }
}
