import { checkAccessibilityPermission } from "tauri-plugin-macos-permissions-api";
import { commands } from "@/bindings";

export async function hasMacOSAccessibilityPermission(): Promise<boolean> {
  try {
    if (await checkAccessibilityPermission()) {
      return true;
    }
  } catch (error) {
    console.warn("Plugin accessibility check failed:", error);
  }

  try {
    const result = await commands.initializeEnigo();
    if (result.status === "ok") {
      const shortcutsResult = await commands.initializeShortcuts();
      if (shortcutsResult.status === "error") {
        console.warn(
          "Accessibility granted, but shortcut initialization failed:",
          shortcutsResult.error,
        );
      }
      return true;
    }

    console.warn("Backend accessibility probe failed:", result.error);
    return false;
  } catch (error) {
    console.warn("Backend accessibility probe threw:", error);
    return false;
  }
}
