import { useEffect, useState, useRef } from "react";
import { toast, Toaster } from "sonner";
import { listen } from "@tauri-apps/api/event";
import { platform } from "@tauri-apps/plugin-os";
import { checkMicrophonePermission } from "tauri-plugin-macos-permissions-api";
import { RecordingErrorEvent } from "./lib/types/events";
import "./App.css";
import AccessibilityPermissions from "./components/AccessibilityPermissions";
import { AccessibilityOnboarding } from "./components/onboarding";
import { Sidebar, SidebarSection, SECTIONS_CONFIG } from "./components/Sidebar";
import { useSettingsStore } from "./stores/settingsStore";
import { commands } from "@/bindings";
import { hasMacOSAccessibilityPermission } from "@/lib/macosAccessibility";

type OnboardingStep = "accessibility" | "done";

const renderSettingsContent = (section: SidebarSection) => {
  const ActiveComponent =
    SECTIONS_CONFIG[section]?.component || SECTIONS_CONFIG.general.component;
  return <ActiveComponent />;
};

function App() {
  const [onboardingStep, setOnboardingStep] = useState<OnboardingStep | null>(
    null,
  );
  const [currentSection, setCurrentSection] =
    useState<SidebarSection>("general");
  const refreshAudioDevices = useSettingsStore(
    (state) => state.refreshAudioDevices,
  );
  const refreshOutputDevices = useSettingsStore(
    (state) => state.refreshOutputDevices,
  );
  const hasCompletedPostOnboardingInit = useRef(false);

  useEffect(() => {
    checkOnboardingStatus();
  }, []);

  // Initialize Enigo, shortcuts, and refresh audio devices when main app loads
  useEffect(() => {
    if (onboardingStep === "done" && !hasCompletedPostOnboardingInit.current) {
      hasCompletedPostOnboardingInit.current = true;
      Promise.all([
        commands.initializeEnigo(),
        commands.initializeShortcuts(),
      ]).catch((e) => {
        console.warn("Failed to initialize:", e);
      });
      refreshAudioDevices();
      refreshOutputDevices();
    }
  }, [onboardingStep, refreshAudioDevices, refreshOutputDevices]);

  // Listen for recording errors from the backend and show a toast
  useEffect(() => {
    const unlisten = listen<RecordingErrorEvent>("recording-error", (event) => {
      const { error_type, detail } = event.payload;

      if (error_type === "microphone_permission_denied") {
        const currentPlatform = platform();
        const description =
          currentPlatform === "macos"
            ? "Grant Handy access in System Settings > Privacy & Security > Microphone and Accessibility."
            : currentPlatform === "windows"
              ? "Grant Handy microphone access in Windows Privacy settings."
              : "Grant Handy microphone access in your system settings.";
        toast.error("Microphone permission required", { description });
      } else if (error_type === "no_input_device") {
        toast.error("No microphone found", {
          description: "Connect or enable a microphone, then try again.",
        });
      } else {
        toast.error(`Recording failed: ${detail ?? "Unknown error"}`);
      }
    });
    return () => {
      unlisten.then((fn) => fn()).catch(console.error);
    };
  }, []);

  // Listen for paste failures and show a toast.
  // The technical error detail is logged to handy.log on the Rust side
  // (see actions.rs `error!("Failed to paste transcription: ...")`),
  // so we show a localized, user-friendly message here instead of the raw error.
  useEffect(() => {
    const unlisten = listen("paste-error", () => {
      toast.error("Paste failed", {
        description:
          "Handy could not type the transcription into the active app.",
      });
    });
    return () => {
      unlisten.then((fn) => fn()).catch(console.error);
    };
  }, []);

  const revealMainWindowForPermissions = async () => {
    try {
      await commands.showMainWindowCommand();
    } catch (e) {
      console.warn("Failed to show main window for permission onboarding:", e);
    }
  };

  const checkOnboardingStatus = async () => {
    try {
      const currentPlatform = platform();

      if (currentPlatform === "macos") {
        try {
          const [hasAccessibility, hasMicrophone] = await Promise.all([
            hasMacOSAccessibilityPermission(),
            checkMicrophonePermission(),
          ]);
          if (!hasAccessibility || !hasMicrophone) {
            await revealMainWindowForPermissions();
            setOnboardingStep("accessibility");
            return;
          }
        } catch (e) {
          console.warn("Failed to check macOS permissions:", e);
        }
      }

      if (currentPlatform === "windows") {
        try {
          const microphoneStatus =
            await commands.getWindowsMicrophonePermissionStatus();
          if (
            microphoneStatus.supported &&
            microphoneStatus.overall_access === "denied"
          ) {
            await revealMainWindowForPermissions();
            setOnboardingStep("accessibility");
            return;
          }
        } catch (e) {
          console.warn("Failed to check Windows microphone permissions:", e);
        }
      }

      setOnboardingStep("done");
    } catch (error) {
      console.error("Failed to check onboarding status:", error);
      setOnboardingStep("accessibility");
    }
  };

  const handleAccessibilityComplete = () => {
    setOnboardingStep("done");
  };

  // Still checking onboarding status
  if (onboardingStep === null) {
    return null;
  }

  if (onboardingStep === "accessibility") {
    return <AccessibilityOnboarding onComplete={handleAccessibilityComplete} />;
  }

  return (
    <div className="h-screen flex flex-col select-none cursor-default">
      <Toaster
        theme="system"
        toastOptions={{
          unstyled: true,
          classNames: {
            toast:
              "bg-background border border-mid-gray/20 rounded-lg shadow-lg px-4 py-3 flex items-center gap-3 text-sm",
            title: "font-medium",
            description: "text-mid-gray",
          },
        }}
      />
      {/* Main content area that takes remaining space */}
      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          activeSection={currentSection}
          onSectionChange={setCurrentSection}
        />
        {/* Scrollable content area */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <div className="flex flex-col items-center p-4 gap-4">
              <AccessibilityPermissions />
              {renderSettingsContent(currentSection)}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
