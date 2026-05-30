import React from "react";
import { Slider } from "../ui/Slider";
import { useSettings } from "../../hooks/useSettings";

export const KeyDelaySlider: React.FC = () => {
  const { getSetting, updateSetting } = useSettings();
  const keyDelayMs = getSetting("key_delay_ms") ?? 10;

  return (
    <Slider
      value={keyDelayMs}
      onChange={(value: number) => updateSetting("key_delay_ms", value)}
      min={0}
      max={100}
      step={1}
      label="Keystroke delay"
      description="Delay between each keystroke when pasting text. Increase if characters are being dropped."
      descriptionMode="tooltip"
      grouped
      formatValue={(value) => `${Math.round(value)}ms`}
    />
  );
};
