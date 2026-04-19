import React from "react";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { MistralTranscriptionSettings } from "../MistralTranscriptionSettings";

export const ModelSettingsCard: React.FC = () => {
  return (
    <SettingsGroup title="Transcription">
      <MistralTranscriptionSettings />
    </SettingsGroup>
  );
};
