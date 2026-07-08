#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaidProfile {
    Single = 0,
    Raid0 = 1,
    Raid1 = 2,
    Raid5 = 5,
    Raid6 = 6,
    Raid10 = 10,
}

pub struct RaidEngine {
    pub profile: RaidProfile,
    pub chunk_size_blocks: u32,
    pub num_devices: usize,
}

impl RaidEngine {
    pub fn new(profile: RaidProfile, chunk_size_blocks: u32, num_devices: usize) -> Self {
        Self {
            profile,
            chunk_size_blocks,
            num_devices,
        }
    }
    
    pub fn map_read(&self, logical_block: u64) -> Vec<(usize, u64)> {
        match self.profile {
            RaidProfile::Single => vec![(0, logical_block)],
            RaidProfile::Raid0 => {
                let stripe_idx = logical_block / self.chunk_size_blocks as u64;
                let offset_in_stripe = logical_block % self.chunk_size_blocks as u64;
                let dev_idx = (stripe_idx % self.num_devices as u64) as usize;
                let physical_block = (stripe_idx / self.num_devices as u64) * self.chunk_size_blocks as u64 + offset_in_stripe;
                vec![(dev_idx, physical_block)]
            }
            RaidProfile::Raid1 => {
                // In a mirror, we can read from any device. For this simple map function, we just return the first device.
                // A load balancer would choose the actual device.
                vec![(0, logical_block)]
            }
            _ => vec![(0, logical_block)], // Placeholder for RAID5/6/10
        }
    }

    pub fn map_write(&self, logical_block: u64) -> Vec<(usize, u64)> {
        match self.profile {
            RaidProfile::Single => vec![(0, logical_block)],
            RaidProfile::Raid0 => {
                let stripe_idx = logical_block / self.chunk_size_blocks as u64;
                let offset_in_stripe = logical_block % self.chunk_size_blocks as u64;
                let dev_idx = (stripe_idx % self.num_devices as u64) as usize;
                let physical_block = (stripe_idx / self.num_devices as u64) * self.chunk_size_blocks as u64 + offset_in_stripe;
                vec![(dev_idx, physical_block)]
            }
            RaidProfile::Raid1 => {
                // Write to all devices
                (0..self.num_devices).map(|dev_idx| (dev_idx, logical_block)).collect()
            }
            _ => vec![(0, logical_block)], // Placeholder for RAID5/6/10
        }
    }
}
