use core::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextureRange {
    pub mip_level: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub depth_or_array_layers: Option<u32>,
    pub offset: u64,
}

pub struct TextureDescriptor {
    pub label: Option<String>,
    pub size: wgpu::Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: wgpu::TextureDimension,
    pub format: Option<wgpu::TextureFormat>,
    pub usage: wgpu::TextureUsages,
}

impl TextureDescriptor {
    fn raw_desc(&self, output_format: wgpu::TextureFormat) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            label: self.label.as_deref(),
            size: self.size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format.unwrap_or(output_format),
            usage: self.usage,
        }
    }
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            size: Default::default(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: None,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }
    }
}

pub struct Texture {
    sampler: wgpu::Sampler,
    texture_desc: TextureDescriptor,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    bytes_per_row: Option<NonZeroU32>,
    rows_per_image: Option<NonZeroU32>,
}

impl Texture {
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler_desc: &wgpu::SamplerDescriptor,
        texture_desc: TextureDescriptor,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let raw_texture_desc = texture_desc.raw_desc(output_format);
        let sampler = Self::rebuild_sampler(device, sampler_desc);
        let (texture, view, bytes_per_row, rows_per_image) =
            Self::rebuild_texture(device, &raw_texture_desc);
        let bind_group = Self::rebuild_bind_group(
            device,
            bind_group_layout,
            &sampler,
            &view,
            raw_texture_desc.label,
        );
        Self {
            sampler,
            texture_desc,
            texture,
            view,
            bind_group,
            bytes_per_row,
            rows_per_image,
        }
    }

    fn rebuild_sampler(device: &wgpu::Device, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        device.create_sampler(desc)
    }

    fn rebuild_texture(
        device: &wgpu::Device,
        raw_desc: &wgpu::TextureDescriptor,
    ) -> (
        wgpu::Texture,
        wgpu::TextureView,
        Option<NonZeroU32>,
        Option<NonZeroU32>,
    ) {
        let raw = device.create_texture(raw_desc);
        let view = raw.create_view(&Default::default());
        let format_info = raw_desc.format.describe();
        let bytes_per_row = NonZeroU32::new(
            raw_desc.size.width / format_info.block_dimensions.0 as u32
                * format_info.block_size as u32,
        );
        let rows_per_image = if raw_desc.dimension != wgpu::TextureDimension::D3
            && raw_desc.size.depth_or_array_layers > 1
        {
            NonZeroU32::new(raw_desc.size.height / format_info.block_dimensions.1 as u32)
        } else {
            None
        };
        (raw, view, bytes_per_row, rows_per_image)
    }

    fn rebuild_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        view: &wgpu::TextureView,
        label: wgpu::Label,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        })
    }

    pub fn set_data(&mut self, queue: &wgpu::Queue, data: &[u8], range: TextureRange) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: range.mip_level,
                origin: wgpu::Origin3d {
                    x: range.x,
                    y: range.y,
                    z: range.z,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: range.offset,
                bytes_per_row: self.bytes_per_row,
                rows_per_image: self.rows_per_image,
            },
            wgpu::Extent3d {
                width: range.width.unwrap_or(self.texture_desc.size.width),
                height: range.height.unwrap_or(self.texture_desc.size.height),
                depth_or_array_layers: range
                    .depth_or_array_layers
                    .unwrap_or(self.texture_desc.size.depth_or_array_layers),
            },
        );
    }

    fn rebuild_texture_from_current_desc(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        output_format: wgpu::TextureFormat,
    ) {
        let raw_texture_desc = self.texture_desc.raw_desc(output_format);
        let (texture, view, bytes_per_row, rows_per_image) =
            Self::rebuild_texture(device, &raw_texture_desc);
        self.bind_group = Self::rebuild_bind_group(
            device,
            bind_group_layout,
            &self.sampler,
            &view,
            raw_texture_desc.label,
        );
        self.texture = texture;
        self.view = view;
        self.bytes_per_row = bytes_per_row;
        self.rows_per_image = rows_per_image;
    }

    pub fn rebuild_with_texture_desc(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        desc: TextureDescriptor,
        output_format: wgpu::TextureFormat,
    ) {
        self.texture_desc = desc;
        self.rebuild_texture_from_current_desc(device, bind_group_layout, output_format);
    }

    pub fn change_swapchain_format(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        output_format: wgpu::TextureFormat,
    ) {
        if self.texture_desc.format.is_none() {
            self.rebuild_texture_from_current_desc(device, bind_group_layout, output_format);
        }
    }
}
