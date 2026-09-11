use bevy::{
    asset::{LoadContext, UntypedAssetId, VisitAssetDependencies},
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};
use ron_asset_manager::{Shandle, prelude::RonAsset};
use serde::Deserialize;

#[derive(Clone, Deserialize, Default)]
pub struct TextureReference {
    pub path: String,
    #[serde(default)]
    pub sampling: TextureSampling,
    #[serde(skip)]
    pub handle: Shandle<Image>,
}

#[derive(Clone, Copy, Default, Deserialize)]
pub enum TextureSampling {
    #[default]
    Default,
    Linear,
    Nearest,
}

impl TextureReference {
    pub fn handle(&self) -> &Handle<Image> {
        &self.handle.handle
    }

    fn load(&mut self, load_context: &mut LoadContext<'_>) {
        let sampling = self.sampling;
        self.handle.handle = load_context
            .load_builder()
            .with_settings(move |settings: &mut bevy::image::ImageLoaderSettings| {
                settings.sampler = match sampling {
                    TextureSampling::Default => ImageSampler::Default,
                    TextureSampling::Linear => {
                        ImageSampler::Descriptor(ImageSamplerDescriptor::linear())
                    }
                    TextureSampling::Nearest => {
                        ImageSampler::Descriptor(ImageSamplerDescriptor::nearest())
                    }
                };
            })
            .load(self.path.clone())
    }
}

impl RonAsset for TextureReference {
    fn load_assets(&mut self, load_context: &mut LoadContext<'_>) {
        self.load(load_context);
        self.handle.path = self.path.clone();
    }
}

impl VisitAssetDependencies for TextureReference {
    fn visit_dependencies(&self, visit: &mut impl FnMut(UntypedAssetId)) {
        self.handle.handle.visit_dependencies(visit);
    }
}
