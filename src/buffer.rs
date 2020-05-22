use gfx_hal::{adapter::PhysicalDevice, device::Device, MemoryTypeId};

pub unsafe fn new_buffer<B: gfx_hal::Backend>(
    device: &B::Device,
    physical_device: &B::PhysicalDevice,
    buffer_len: usize,
    usage: gfx_hal::buffer::Usage,
    properties: gfx_hal::memory::Properties,
) -> (B::Memory, B::Buffer) {
    // This creates a handle to a buffer. The `buffer_len` is in bytes,
    // and the usage states what kind of buffer it is.
    let mut buffer = device
        .create_buffer(buffer_len as u64, usage)
        .expect("Failed to create buffer");

    // determine memory type and requirements
    let req = device.get_buffer_requirements(&buffer);

    let memory_types = physical_device.memory_properties().memory_types;

    let memory_type = memory_types
        .iter()
        .enumerate()
        .find(|(id, mem_type)| {
            let type_supported = req.type_mask & (1_u64 << id) != 0;
            type_supported && mem_type.properties.contains(properties)
        })
        .map(|(id, _ty)| MemoryTypeId(id))
        .expect("No compatible memory type available");

    // allocate memory for the buffer
    let buffer_memory = device
        .allocate_memory(memory_type, req.size)
        .expect("Failed to allocate buffer memory");

    // bind the memory to the buffer
    device
        .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
        .expect("Failed to bind buffer memory");

    (buffer_memory, buffer)
}
