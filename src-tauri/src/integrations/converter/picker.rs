use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, FOS_ALLOWMULTISELECT,
            FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
        },
    },
};

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub fn pick(kind: &str) -> Result<Vec<String>, String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        let _apartment = Apartment;
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        let mut options = dialog.GetOptions().map_err(|e| e.to_string())? | FOS_FORCEFILESYSTEM;
        let (title, filter) = match kind {
            "files" => {
                options |= FOS_ALLOWMULTISELECT;
                ("Choose files to convert", "*.jpg;*.jpeg;*.png;*.webp;*.heic;*.heif;*.tif;*.tiff;*.bmp;*.gif;*.avif;*.pdf;*.doc;*.docx;*.xls;*.xlsx;*.ppt;*.pptx;*.txt;*.md;*.csv")
            }
            "executable" => ("Locate Filee's filee-cli.exe", "filee-cli.exe"),
            "folder" => {
                options |= FOS_PICKFOLDERS;
                ("Choose output folder", "*")
            }
            _ => return Err("Unknown converter picker.".into()),
        };
        dialog.SetOptions(options).map_err(|e| e.to_string())?;
        dialog
            .SetTitle(&HSTRING::from(title))
            .map_err(|e| e.to_string())?;
        let label = HSTRING::from(if kind == "executable" {
            "Filee CLI"
        } else {
            "Supported files"
        });
        let pattern = HSTRING::from(filter);
        if kind != "folder" {
            dialog
                .SetFileTypes(&[COMDLG_FILTERSPEC {
                    pszName: PCWSTR(label.as_ptr()),
                    pszSpec: PCWSTR(pattern.as_ptr()),
                }])
                .map_err(|e| e.to_string())?;
        }
        if let Err(error) = dialog.Show(None) {
            if error.code().0 as u32 == 0x800704c7 {
                return Ok(Vec::new());
            }
            return Err(format!("Could not open the file picker: {error}"));
        }
        let items = dialog.GetResults().map_err(|e| e.to_string())?;
        let count = items.GetCount().map_err(|e| e.to_string())?;
        if count > 20 {
            return Err("Choose at most 20 files.".into());
        }
        let mut paths = Vec::new();
        for index in 0..count {
            let raw = items
                .GetItemAt(index)
                .and_then(|item| item.GetDisplayName(SIGDN_FILESYSPATH))
                .map_err(|e| e.to_string())?;
            let result = raw.to_string().map_err(|e| e.to_string());
            CoTaskMemFree(Some(raw.as_ptr().cast()));
            paths.push(result?);
        }
        Ok(paths)
    }
}
