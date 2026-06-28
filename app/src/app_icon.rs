use objc2::rc::Retained;
use objc2_app_kit::{NSBitmapImageRep, NSImage, NSWorkspace};
use objc2_foundation::{NSAutoreleasePool, NSString};
use objc2_uniform_type_identifiers::UTType;

pub struct AppIcon {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: Vec<u8>,
}

pub struct FolderIcon {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: Vec<u8>,
}

pub fn fetch_app_icon(bundle_id: &str) -> Option<AppIcon> {
    unsafe {
        let pool = NSAutoreleasePool::new();

        // Corrected: sharedWorkspace takes NO arguments in modern objc2
        let workspace = NSWorkspace::sharedWorkspace();
        let ns_bundle_id = NSString::from_str(bundle_id);

        // Fetch the URL and path safely without mixing up Retained types
        let app_url = workspace.URLForApplicationWithBundleIdentifier(&ns_bundle_id)?;
        let app_path = app_url.path()?;

        // Extract the image. We assign to Option first so we can use the `?` operator safely.
        let ns_image: Retained<NSImage> = workspace.iconForFile(&app_path);

        // Force Apple to extract the raw uncompressed bitmap representation
        let tiff_bytes = ns_image.TIFFRepresentation()?;
        let bitmap_rep = NSBitmapImageRep::imageRepWithData(&tiff_bytes)?;

        // Gather metrics directly out of Apple's graphics buffer
        let width = bitmap_rep.pixelsWide() as usize;
        let height = bitmap_rep.pixelsHigh() as usize;
        let bytes_per_row = bitmap_rep.bytesPerRow() as usize;
        let bits_per_pixel = bitmap_rep.bitsPerPixel() as usize;
        let bytes_per_pixel = bits_per_pixel / 8;

        let bitmap_data_ptr = bitmap_rep.bitmapData();
        if bitmap_data_ptr.is_null() {
            return None;
        }

        // Formulate a safe slice out of the core memory block and map to Vector
        let total_allocated_bytes = bytes_per_row * height;
        let raw_slice = std::slice::from_raw_parts(bitmap_data_ptr, total_allocated_bytes);
        let mut rgba_bytes = Vec::with_capacity(width * height * bytes_per_pixel);
        for row in 0..height {
            let start = row * bytes_per_row;
            let end = start + (width * bytes_per_pixel);
            rgba_bytes.extend_from_slice(&raw_slice[start..end]);
        }

        drop(pool);

        Some(AppIcon {
            width: width as u32,
            height: height as u32,
            rgba_bytes,
        })
    }
}

/// Retrieves the raw uncompressed pixel data of the standard default Finder folder icon.
pub fn fetch_finder_folder_icon() -> Option<FolderIcon> {
    unsafe {
        let pool = NSAutoreleasePool::new();

        // Grab the global type-safe workspace singleton instance
        let workspace = NSWorkspace::sharedWorkspace();

        // Build the identifier string representation ("public.folder")
        let type_string = NSString::from_str("public.folder");

        // Create the type token wrapper via Uniform Type Identifiers
        let content_type = UTType::typeWithIdentifier(&type_string)?;

        // Force Apple to export the bitmap representation buffer
        let ns_image: Retained<NSImage> = workspace.iconForContentType(&content_type);
        let tiff_bytes = ns_image.TIFFRepresentation()?;
        let bitmap_rep = NSBitmapImageRep::imageRepWithData(&tiff_bytes)?;

        let width = bitmap_rep.pixelsWide() as usize;
        let height = bitmap_rep.pixelsHigh() as usize;
        let bytes_per_row = bitmap_rep.bytesPerRow() as usize;
        let bits_per_pixel = bitmap_rep.bitsPerPixel() as usize;
        let bytes_per_pixel = bits_per_pixel / 8;

        let bitmap_data_ptr = bitmap_rep.bitmapData();
        if bitmap_data_ptr.is_null() {
            return None;
        }

        // Wrap raw memory block natively into a Rust Vector
        let total_allocated_bytes = bytes_per_row * height;
        let raw_slice = std::slice::from_raw_parts(bitmap_data_ptr, total_allocated_bytes);
        let mut rgba_bytes = Vec::with_capacity(width * height * bytes_per_pixel);
        for row in 0..height {
            let start = row * bytes_per_row;
            let end = start + (width * bytes_per_pixel);
            rgba_bytes.extend_from_slice(&raw_slice[start..end]);
        }

        drop(pool);

        Some(FolderIcon {
            width: width as u32,
            height: height as u32,
            rgba_bytes,
        })
    }
}
