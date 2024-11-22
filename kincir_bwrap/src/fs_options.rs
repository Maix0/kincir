/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   fs_options.rs                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/22 22:10:33 by maiboyer          #+#    #+#             */
/*   Updated: 2024/11/22 22:36:05 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::CowStr;
use std::os::fd::AsRawFd;

macro_rules! vec_size {
    ($default_size:literal) => {
        0usize + $default_size
    };

    (@perm: $default_size:literal, $permission:ident) => {
        $default_size + if $permission.is_some() { 2usize } else { 0 }
    };
    (@size: $default_size:literal, $size:ident) => {
        $default_size + if $size.is_some() { 2usize } else { 0 }
    };
    (@permsize: $default_size:literal, $permission:ident ,$size:ident) => {
        $default_size
            + if $size.is_some() { 2usize } else { 0 }
            + if $permission.is_some() { 2usize } else { 0 }
    };
}

macro_rules! vec_append {
    (@perm: &mut $vec:ident, $permission:ident) => {
        if let Some(p) = $permission.as_ref() {
            $vec.push(CowStr::from("--perm"));
            $vec.push(CowStr::from(format!("{p:o}")));
        }
    };
    (@size: &mut $vec:ident, $permission:ident) => {
        if let Some(s) = $permission.as_ref() {
            $vec.push(CowStr::from("--size"));
            $vec.push(CowStr::from(s.to_string()));
        }
    };
}

macro_rules! bwrap_flag {
    (@none: $flag:literal) => {
        CowStr::from(concat!("--", $flag))
    };
    (@ro: $flag:literal, $bool:expr) => {
        if $bool {
            CowStr::from(concat!("--ro-", $flag))
        } else {
            CowStr::from(concat!("--", $flag))
        }
    };
    (@try: $flag:literal, $bool:expr) => {
        if $bool {
            CowStr::from(concat!("--", $flag, "-try"))
        } else {
            CowStr::from(concat!("--", $flag, ""))
        }
    };
    (@rotry: $flag:literal, $bool_ro:expr, $bool_try:expr) => {
        CowStr::from(match ($bool_ro, $bool_try) {
            (true, true) => concat!("--ro-", $flag, "-try"),
            (false, true) => concat!("--", $flag, "-try"),
            (true, false) => concat!("--ro", $flag, ""),
            (false, false) => concat!("--", $flag, ""),
        })
    };
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FsOptions {
    /// the equivalent of `--bind` (with the `read_only` or try modifier if set)
    /// This means that the file/directory will be visible from inside the sandbox at the given
    /// path (destination). it will be backed up by the directory/file at `source` on the host
    /// (outside the sandbox)
    Bind {
        /// This would mean `--ro-bind` if set
        /// This makes sure that the sandbox can't write the the files under the bind, even if the
        /// permission would allow it
        read_only: bool,
        /// Where does the bind points while looking outside of the sandbox
        source: CowStr,
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
        /// thie allow the bind to silently ignore if the source path doesn't exists
        try_: bool,
    },
    /// the equivalent of `--bind-dev` (with try modifier if set)
    /// This means that the file/directory will be visible from inside the sandbox at the given
    /// path (destination). This allows the use of device files through the bind
    DevBind {
        /// Where does the bind points while looking outside of the sandbox
        source: CowStr,
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
        /// thie allow the bind to silently ignore if the source path doesn't exists
        try_: bool,
    },
    /// the equivalent of `--proc-dev` (with try modifier if set)
    /// This means that the file/directory will be visible from inside the sandbox at the given
    /// path (destination). This allows the use of procfs through the bind
    ProcBind {
        /// Where does the bind points while looking outside of the sandbox
        source: CowStr,
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
        /// thie allow the bind to silently ignore if the source path doesn't exists
        try_: bool,
    },
    /// Create a new devfs at the specifed path
    Dev {
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
    },
    /// Create a new procfs at the specifed path
    Proc {
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
    },
    /// Create a new mqueue at the specifed path
    MQueue {
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
    },
    /// Create a new directory at the specifed path
    ///
    /// # Note
    /// if the directory already exists, then the permission are not modifed even if the permission
    /// are not None, please use a [`FsOptions::Chmod`] for that
    Dir {
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
    },
    /// Will create a tempfs that will live inside the sandbox at the destination path
    /// if no size are set it will use bwrap's default size
    TempFs {
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
        /// the maximum size of the tempfs
        size: Option<usize>,
    },
    /// This will create a symlink inside the sandbox.
    /// # note
    /// the behaviour changes a bit between bwrap version:
    ///  - before 0.9.0 if DEST already existed, this would be treated as an error (even if its target was identical to SRC)
    ///  - since 0.9.9 it is not considered to be an error if DEST already exists as a symbolic link and its target is exactly SRC
    ///
    ///  this means that depending on the version of bwrap your system uses, it *could* encounters
    ///  error where a more recent system would not.
    Symlink {
        /// Where does the bind points while looking outside of the sandbox
        source: CowStr,
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
    },
    File {
        /// The filedescriptor that will be used in the `--file` flag. Please check the manpage of
        /// `bwrap(1)` to see more information about it
        source: std::os::fd::OwnedFd,
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
    },
    Data {
        /// The filedescriptor that will be used in the `--data` flag. Please check the manpage of
        /// `bwrap(1)` to see more information about it
        source: std::os::fd::OwnedFd,
        /// Where does the bind lives while inside of the sandbox
        destination: CowStr,
        /// if set to Some value, what will be the permission of the bind inside the sandbox
        permission: Option<u64>,
        /// This would mean `--ro-data` if set
        /// This makes sure that the sandbox can't write the the files under the bind, even if the
        /// permission would allow it
        read_only: bool,
    },

    /// Change the permission of an existing file inside the sandbox
    Chmod {
        /// Which file/directory/path to change the permission
        destination: CowStr,
        /// Change the permission of the directory or file that already exists, but only while
        /// looking from inside the sandbox
        permission: u64,
    },
}

impl FsOptions {
    #[must_use] pub fn to_option(&self) -> impl IntoIterator<Item = CowStr> {
        match self {
            Self::Chmod {
                destination,
                permission,
            } => vec![
                CowStr::from("--chmod"),
                CowStr::from(format!("{permission:o}")),
                destination.clone(),
            ],
            Self::Data {
                source,
                destination,
                permission,
                read_only,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@ro: "data-bind", *read_only));
                v.push(source.as_raw_fd().to_string().into());
                v.push(destination.clone());
                v
            }
            Self::File {
                source,
                destination,
                permission,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@none: "file"));
                v.push(source.as_raw_fd().to_string().into());
                v.push(destination.clone());
                v
            }
            Self::Symlink {
                source,
                destination,
            } => {
                vec![
                    bwrap_flag!(@none: "symlink"),
                    source.clone(),
                    destination.clone(),
                ]
            }
            Self::TempFs {
                destination,
                size,
                permission,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@permsize: 3, permission, size));
                vec_append!(@perm: &mut v, permission);
                vec_append!(@size: &mut v, size);
                v.push(bwrap_flag!(@none: "tmpfs"));
                v.push(destination.clone());
                v
            }
            Self::Dir {
                destination,
                permission,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@none: "dir"));
                v.push(destination.clone());
                v
            }
            Self::MQueue {
                destination,
                permission,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@none: "mqueue"));
                v.push(destination.clone());
                v
            }
            FsOptions::Bind {
                read_only,
                source,
                destination,
                permission,
                try_,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@rotry: "bind", *read_only ,*try_));
                v.push(source.clone());
                v.push(destination.clone());
                v
            }
            FsOptions::DevBind {
                source,
                destination,
                permission,
                try_,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@try: "dev-bind", *try_));
                v.push(source.clone());
                v.push(destination.clone());
                v
            }
            FsOptions::ProcBind {
                source,
                destination,
                permission,
                try_,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@try: "proc-bind", *try_));
                v.push(source.clone());
                v.push(destination.clone());
                v
            }
            FsOptions::Dev {
                destination,
                permission,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@none: "dev"));
                v.push(destination.clone());
                v
            }
            FsOptions::Proc {
                destination,
                permission,
            } => {
                let mut v = Vec::with_capacity(vec_size!(@perm: 3, permission));
                vec_append!(@perm: &mut v, permission);
                v.push(bwrap_flag!(@none: "proc"));
                v.push(destination.clone());
                v
            }
        }
    }
}
