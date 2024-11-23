use bitflags::bitflags;

use crate::CowStr;

bitflags! {
    #[derive(Debug, Clone,Copy, Eq, PartialEq, Hash)]
    pub struct NamespaceFlags: u32 {
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
    }
}

impl NamespaceFlags {
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

    #[must_use]
    pub fn to_options(mut self) -> impl IntoIterator<Item = CowStr> {
        self = self.sanitize();
        let mut v = Vec::<CowStr>::with_capacity(self.bits().count_ones() as usize);
        for flag in self.iter() {
            v.push(CowStr::from(match flag {
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
                _ => continue,
            }));
        }

        v
    }
}

#[cfg(test)]
mod test {
    use super::NamespaceFlags as F;
    use super::*;

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
            flags.to_options().into_iter().collect::<Vec<_>>(),
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
            flags.to_options().into_iter().collect::<Vec<_>>(),
            vec!["--unshare-user", "--share-net"]
        );
    }
}
