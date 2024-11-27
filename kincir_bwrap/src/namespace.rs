use bitflags::bitflags;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default)]
pub struct NsOptions {
    pub flags: NsFlags,
    gid: Option<std::ffi::c_int>,
    uid: Option<std::ffi::c_int>,
    hostname: Option<OsString>,
    cwd: Option<PathBuf>,
}

impl NsOptions {
    pub fn set_cwd(&mut self, cwd: impl AsRef<Path>) {
        self.cwd = Some(cwd.as_ref().to_path_buf());
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn cwd(&mut self, cwd: Option<impl AsRef<Path>>) {
        self.cwd = cwd.map(|cwd| cwd.as_ref().to_path_buf());
    }

    pub fn unset_cwd(&mut self) {
        self.cwd = None;
    }

    pub fn set_hostname(&mut self, hostname: impl AsRef<OsStr>) {
        self.hostname = Some(hostname.as_ref().into());
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn hostname(&mut self, hostname: Option<impl AsRef<OsStr>>) {
        self.hostname = hostname.as_ref().map(Into::into);
    }

    pub fn unset_hostname(&mut self) {
        self.hostname = None;
    }

    pub fn set_uid(&mut self, uid: impl Into<std::ffi::c_int>) {
        self.uid = Some(uid.into());
    }

    pub fn uid(&mut self, uid: Option<impl Into<std::ffi::c_int>>) {
        self.uid = uid.map(Into::into);
    }

    pub fn unset_uid(&mut self) {
        self.uid = None;
    }

    pub fn set_gid(&mut self, uid: impl Into<std::ffi::c_int>) {
        self.gid = Some(uid.into());
    }

    pub fn gid(&mut self, gid: Option<impl Into<std::ffi::c_int>>) {
        self.gid = gid.map(Into::into);
    }

    pub fn unset_gid(&mut self) {
        self.gid = None;
    }
}

impl NsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sanitize_flags(&mut self) {
        if self.gid.is_some() {
            self.flags.set(NsFlags::USER, true);
        }
        if self.uid.is_some() {
            self.flags.set(NsFlags::USER, true);
        }
        if self.hostname.is_some() {
            self.flags.set(NsFlags::UTS, true);
        }

        self.flags = self.flags.sanitize();
    }

    pub fn to_options(&mut self) -> impl Iterator<Item = OsString> {
        self.sanitize_flags();
        let mut v = self.flags.to_options().collect::<Vec<_>>();

        if let Some(&gid) = self.gid.as_ref() {
            v.push(OsString::from("--gid"));
            v.push(OsString::from(gid.to_string()));
        }
        if let Some(&uid) = self.uid.as_ref() {
            v.push(OsString::from("--uid"));
            v.push(OsString::from(uid.to_string()));
        }
        if let Some(hostname) = self.hostname.as_ref() {
            v.push(OsString::from("--hostname"));
            v.push(OsString::from(hostname));
        }
        if let Some(cwd) = self.cwd.as_ref() {
            v.push(OsString::from("--chdir"));
            v.push(OsString::from(cwd));
        }

        v.into_iter()
    }
}

impl NsFlags {
    /// Sanitize the flags such that some know unwanted combination are filtered out.
    ///
    ///
    /// Currently this includes:
    /// - the `_TRY` variants are removed if the non `_TRY` variants are present
    ///
    /// - all the `--unshare-*` bwrap flags are removed if the [`NamespaceFlags::ALL`] is present
    #[must_use]
    pub fn sanitize(mut self) -> Self {
        if self.contains(Self::ALL) {
            self.remove(
                Self::USER
                    | Self::USER_TRY
                    | Self::IPC
                    | Self::PID
                    | Self::NET
                    | Self::UTS
                    | Self::CGROUPS
                    | Self::CGROUPS_TRY,
            );
        }
        if self.contains(Self::USER) {
            self.remove(Self::USER_TRY);
        }
        if self.contains(Self::CGROUPS) {
            self.remove(Self::CGROUPS_TRY);
        }
        self
    }

    /// This will return a iterator with the bwarp argument that correspond the the given flags.
    ///
    /// Note that the flags are first sanitize using the [`NamespaceFlags::sanitize`] function
    pub fn to_options(mut self) -> impl Iterator<Item = OsString> {
        self = self.sanitize();
        let mut v = Vec::<OsString>::with_capacity(self.bits().count_ones() as usize);
        for flag in self.iter() {
            v.push(OsString::from(match flag {
                Self::CGROUPS_TRY => "--unshare-cgroup-try",
                Self::CGROUPS => "--unshare-cgroup",
                Self::USER_TRY => "--unshare-user-try",
                Self::USER => "--unshare-user",
                Self::SHARE_NET => "--share-net",
                Self::IPC => "--unshare-ipc",
                Self::NET => "--unshare-net",
                Self::PID => "--unshare-pid",
                Self::UTS => "--unshare-uts",
                Self::DISABLE_USER_NS => "--disable-userns",
                Self::ASSERT_DISABLE_USER_NS => "--assert-disable-userns",
                Self::ALL => "--unshare-all",
                Self::NEW_SESSION => "--new-session",
                Self::DIE_WITH_PARENT => "--die-with-parent",
                _ => continue,
            }));
        }

        v.into_iter()
    }
}

bitflags! {
    /// The flags that takes no option that mananage what the sandbox shares/doesn't share with the
    /// host
    #[derive(Debug, Clone,Copy, Eq, PartialEq, Hash, Default)]
    pub struct NsFlags: u32 {
        /// --unshare-user
        const USER = 1 << 0;
        /// --unshare-user-try
        const USER_TRY = 1 << 1;
        /// --unshare-ipc
        const IPC = 1 << 2;
        /// --unshare-pid
        const PID = 1 << 3;
        /// --unshare-net
        const NET = 1 << 4;
        /// --unshare-uts
        const UTS = 1 << 5;
        /// --unshare-cgroup
        const CGROUPS = 1 << 6;
        /// --unshare-cgroup-try
        const CGROUPS_TRY = 1 << 7;


        /// --unshare-all
        const ALL = 1 << 9;

        /// --disable-userns
        const DISABLE_USER_NS = 1 << 10;
        /// --assert-userns-disabled
        const ASSERT_DISABLE_USER_NS = 1 << 11;

        /// --share-net
        const SHARE_NET = 1 << 8;

        /// --die-with-parent
        const DIE_WITH_PARENT = 1 << 12;
        /// --new-session
        const NEW_SESSION = 1 << 13;
    }
}

#[cfg(test)]
mod test {
    use super::NsFlags as F;

    #[test]
    fn try_() {
        let mut flags = F::empty();
        flags.insert(F::CGROUPS);
        flags.insert(F::CGROUPS_TRY);
        assert_eq!(flags, F::CGROUPS | F::CGROUPS_TRY);
        assert_eq!(flags.sanitize(), F::CGROUPS);

        let mut flags = F::empty();
        flags.insert(F::USER);
        flags.insert(F::USER_TRY);
        assert_eq!(flags, F::USER | F::USER_TRY);
        assert_eq!(flags.sanitize(), F::USER);
    }
    #[test]
    fn all() {
        let mut flags = F::empty();
        flags.insert(F::USER);
        flags.insert(F::USER_TRY);
        flags.insert(F::ALL);
        assert_eq!(flags, F::USER | F::USER_TRY | F::ALL);
        assert_eq!(flags.sanitize(), F::ALL);

        let mut flags = F::empty();
        flags.insert(F::USER);
        flags.insert(F::USER_TRY);
        flags.insert(F::SHARE_NET);
        flags.insert(F::ALL);
        assert_eq!(flags, F::USER | F::USER_TRY | F::ALL | F::SHARE_NET);
        assert_eq!(flags.sanitize(), F::ALL | F::SHARE_NET);
    }

    #[test]
    fn args_all() {
        let mut flags = F::empty();
        flags.insert(F::USER);
        flags.insert(F::USER_TRY);
        flags.insert(F::SHARE_NET);
        flags.insert(F::ALL);
        assert_eq!(
            flags.to_options().collect::<Vec<_>>(),
            vec!["--unshare-all", "--share-net"]
        );
    }

    #[test]
    fn args1() {
        let mut flags = F::empty();
        flags.insert(F::USER);
        flags.insert(F::USER_TRY);
        flags.insert(F::SHARE_NET);
        assert_eq!(
            flags.to_options().collect::<Vec<_>>(),
            vec!["--unshare-user", "--share-net"]
        );
    }
}
