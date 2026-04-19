import { checkAccessibilityPermission } from "tauri-plugin-macos-permissions-api";

export async function hasMacOSAccessibilityPermission(): Promise<boolean> {
  try {
    return await checkAccessibilityPermission();
  } catch (error) {
    console.warn("Plugin accessibility check failed:", error);
    return false;
  }
}
