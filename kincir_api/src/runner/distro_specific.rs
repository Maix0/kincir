use super::Runner;

pub type DistroHandler = fn(&mut Runner) -> Result<(), Box<dyn std::error::Error>>;
pub type DistroName = &'static str;

/// This will allow the use of specific handling for some linux distros
/// (specifically nixos since they work in a weird way where the /nix folder will probably be needed)
///
/// This will be a mapping to the `DISTRIB_ID` in `/etc/lsb-release` file to a function that will
/// take a runner and modify some flags/feature/binds to allow smooth execution
pub static DISTRO_HANDLERS: phf::Map<DistroName, DistroHandler> = phf::phf_map! {
    "nixos" => nixos_handling,
    "Ubuntu" => ubuntu_handling,
};

fn nixos_handling(runner: &mut Runner) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

fn ubuntu_handling(runner: &mut Runner) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
