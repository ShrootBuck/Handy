import React from "react";
import { Slider } from "../ui/Slider";
import { useSettings } from "../../hooks/useSettings";

export const VolumeSlider: React.FC = () => {
  const { getSetting, updateSetting } = useSettings();
  const audioFeedbackVolume = getSetting("audio_feedback_volume") ?? 0.5;

  return (
    <Slider
      value={audioFeedbackVolume}
      onChange={(value: number) =>
        updateSetting("audio_feedback_volume", value)
      }
      min={0}
      max={1}
      label="Sound volume"
      description="Controls the start and stop chime volume."
      descriptionMode="tooltip"
      grouped
      formatValue={(value) => `${Math.round(value * 100)}%`}
    />
  );
};
