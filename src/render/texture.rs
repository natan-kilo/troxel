pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(device: &wgpu::Device, path: &str, label: &str) -> (Self, wgpu::CommandBuffer) {
        let image = image::open(path).unwrap();
        let rgba_image = image.as_rgba8().unwrap();

        let image_dim = rgba_image.dimensions();

        let size = create_size(image_dim, 1);

        let texture = device.create_texture(&create_texture_descriptor(
            label,
            size,
            1,
            1,
            1,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        ));

        let buffer = device.create_buffer_with_data(&rgba_image, wgpu::BufferUsage::COPY_SRC);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        encoder.copy_buffer_to_texture(
            buffer_copy_view(&buffer, 0, 4 * image_dim.0, image_dim.1),
            texture_copy_view(&texture, 0, 0, wgpu::Origin3d::ZERO),
            size,
        );

        let cmd_buffer = encoder.finish();

        let view = texture.create_default_view();

        let sampler = device.create_sampler(&default_sampler_descriptor());

        (
            Self {
                texture,
                view,
                sampler,
            },
            cmd_buffer,
        )
    }

    pub fn new_depth(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        label: &str,
    ) -> Self {
        let size = create_size((sc_desc.width, sc_desc.height), 1);

        let texture = device.create_texture(&create_texture_descriptor(
            label,
            size,
            1,
            1,
            1,
            wgpu::TextureDimension::D2,
            DEPTH_FORMAT,
            wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC,
        ));

        let view = texture.create_default_view();

        let sampler = device.create_sampler(&default_sampler_descriptor());

        Self {
            texture,
            view,
            sampler,
        }
    }
}

fn create_size(size: (u32, u32), depth: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth,
    }
}

fn create_texture_descriptor(
    label: &str,
    size: wgpu::Extent3d,
    array_layer_count: u32,
    mip_level_count: u32,
    sample_count: u32,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsage,
) -> wgpu::TextureDescriptor {
    wgpu::TextureDescriptor {
        label: Some(label),
        size,
        array_layer_count,
        mip_level_count,
        sample_count,
        dimension,
        format,
        usage,
    }
}

fn create_sampler_descriptor(
    address_mode: wgpu::AddressMode,
    mag_filter: wgpu::FilterMode,
    min_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: wgpu::CompareFunction,
) -> wgpu::SamplerDescriptor {
    wgpu::SamplerDescriptor {
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter,
        min_filter,
        mipmap_filter,
        lod_min_clamp,
        lod_max_clamp,
        compare,
    }
}

fn default_sampler_descriptor() -> wgpu::SamplerDescriptor {
    wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare: wgpu::CompareFunction::LessEqual,
    }
}

fn buffer_copy_view(
    buffer: &wgpu::Buffer,
    offset: wgpu::BufferAddress,
    bytes_per_row: u32,
    rows_per_image: u32,
) -> wgpu::BufferCopyView {
    wgpu::BufferCopyView {
        buffer,
        offset,
        bytes_per_row,
        rows_per_image,
    }
}

fn texture_copy_view(
    texture: &wgpu::Texture,
    mip_level: u32,
    array_layer: u32,
    origin: wgpu::Origin3d,
) -> wgpu::TextureCopyView {
    wgpu::TextureCopyView {
        texture,
        mip_level,
        array_layer,
        origin,
    }
}
