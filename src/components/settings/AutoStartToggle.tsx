import React from "react";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface AutoStartToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const AutoStartToggle: React.FC<AutoStartToggleProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { getSetting, updateSetting, isUpdating } = useSettings();

    return (
      <ToggleSwitch
        checked={getSetting("autostart_enabled") || false}
        onChange={(enabled) => updateSetting("autostart_enabled", enabled)}
        isUpdating={isUpdating("autostart_enabled")}
        label="Launch at login"
        description="Starts Handy automatically and keeps it hidden in the tray instead of opening a window."
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
