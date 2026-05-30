import { invoke as TAURI_INVOKE } from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

export type Result<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

export type SecretString = string;

export type ShortcutBinding = {
  id: string;
  name: string;
  description: string;
  default_binding: string;
  current_binding: string;
};

export type AppSettings = {
  bindings: Partial<{ [key: string]: ShortcutBinding }>;
  push_to_talk: boolean;
  audio_feedback_volume: number;
  mistral_transcription_api_key: SecretString;
  selected_microphone: string | null;
  selected_output_device: string | null;
  autostart_enabled: boolean;
  key_delay_ms: number;
};

export type AudioDevice = {
  index: string;
  name: string;
  is_default: boolean;
};

export type BindingResponse = {
  success: boolean;
  binding: ShortcutBinding | null;
  error: string | null;
};

export type PermissionAccess = "allowed" | "denied" | "unknown";

export type WindowsMicrophonePermissionStatus = {
  supported: boolean;
  overall_access: PermissionAccess;
  device_access: PermissionAccess;
  app_access: PermissionAccess;
  desktop_app_access: PermissionAccess;
};

export type HistoryEntry = {
  id: number;
  file_name: string;
  timestamp: number;
  saved: boolean;
  title: string;
  transcription_text: string;
};

export type PaginatedHistory = {
  entries: HistoryEntry[];
  has_more: boolean;
};

export type HistoryUpdatePayload =
  | { action: "added"; entry: HistoryEntry }
  | { action: "updated"; entry: HistoryEntry }
  | { action: "deleted"; id: number }
  | { action: "toggled"; id: number };

type EventObject<T> = {
  listen: (
    cb: TAURI_API_EVENT.EventCallback<T>,
  ) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
  once: (
    cb: TAURI_API_EVENT.EventCallback<T>,
  ) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
  emit: null extends T
    ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
    : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

function makeEvent<T>(eventName: string) {
  const builder = (handle?: __WebviewWindow__): EventObject<T> => ({
    listen: (cb) =>
      handle
        ? handle.listen<T>(eventName, cb)
        : TAURI_API_EVENT.listen<T>(eventName, cb),
    once: (cb) =>
      handle
        ? handle.once<T>(eventName, cb)
        : TAURI_API_EVENT.once<T>(eventName, cb),
    emit: (payload?: T) =>
      handle
        ? handle.emit(eventName, payload)
        : TAURI_API_EVENT.emit(eventName, payload),
  });

  return Object.assign(builder, builder()) as EventObject<T> &
    ((handle: __WebviewWindow__) => EventObject<T>);
}

async function invokeResult<T>(command: string, args?: Record<string, unknown>) {
  try {
    return {
      status: "ok" as const,
      data: await TAURI_INVOKE<T>(command, args),
    };
  } catch (e) {
    if (e instanceof Error) throw e;
    return { status: "error" as const, error: e as any };
  }
}

export const commands = {
  changeBinding: (id: string, binding: string) =>
    invokeResult<BindingResponse>("change_binding", { id, binding }),
  resetBinding: (id: string) =>
    invokeResult<BindingResponse>("reset_binding", { id }),
  changePttSetting: (enabled: boolean) =>
    invokeResult<null>("change_ptt_setting", { enabled }),
  changeAudioFeedbackVolumeSetting: (volume: number) =>
    invokeResult<null>("change_audio_feedback_volume_setting", { volume }),
  suspendBinding: (id: string) =>
    invokeResult<null>("suspend_binding", { id }),
  resumeBinding: (id: string) =>
    invokeResult<null>("resume_binding", { id }),
  changeMistralTranscriptionApiKeySetting: (apiKey: string) =>
    invokeResult<null>("change_mistral_transcription_api_key_setting", {
      apiKey,
    }),
  changeKeyDelayMsSetting: (delayMs: number) =>
    invokeResult<null>("change_key_delay_ms_setting", { delayMs }),
  showMainWindowCommand: () =>
    invokeResult<null>("show_main_window_command"),
  cancelOperation: () => TAURI_INVOKE<void>("cancel_operation"),
  getAppSettings: () => invokeResult<AppSettings>("get_app_settings"),
  getDefaultSettings: () =>
    invokeResult<AppSettings>("get_default_settings"),
  changeAutostartSetting: (enabled: boolean) =>
    invokeResult<null>("change_autostart_setting", { enabled }),
  initializeEnigo: () => invokeResult<null>("initialize_enigo"),
  initializeShortcuts: () =>
    invokeResult<null>("initialize_shortcuts"),
  getWindowsMicrophonePermissionStatus: () =>
    TAURI_INVOKE<WindowsMicrophonePermissionStatus>(
      "get_windows_microphone_permission_status",
    ),
  openMicrophonePrivacySettings: () =>
    invokeResult<null>("open_microphone_privacy_settings"),
  getAvailableMicrophones: () =>
    invokeResult<AudioDevice[]>("get_available_microphones"),
  setSelectedMicrophone: (deviceName: string) =>
    invokeResult<null>("set_selected_microphone", { deviceName }),
  getAvailableOutputDevices: () =>
    invokeResult<AudioDevice[]>("get_available_output_devices"),
  setSelectedOutputDevice: (deviceName: string) =>
    invokeResult<null>("set_selected_output_device", { deviceName }),
  isRecording: () => TAURI_INVOKE<boolean>("is_recording"),
  openRecordingsFolder: () =>
    invokeResult<null>("open_recordings_folder"),
  getHistoryEntries: (cursor: number | null, limit: number | null) =>
    invokeResult<PaginatedHistory>("get_history_entries", { cursor, limit }),
  toggleHistoryEntrySaved: (id: number) =>
    invokeResult<null>("toggle_history_entry_saved", { id }),
  getAudioFilePath: (fileName: string) =>
    invokeResult<string>("get_audio_file_path", { fileName }),
  deleteHistoryEntry: (id: number) =>
    invokeResult<null>("delete_history_entry", { id }),
  retryHistoryEntryTranscription: (id: number) =>
    invokeResult<null>("retry_history_entry_transcription", { id }),
};

export const events = {
  historyUpdatePayload: makeEvent<HistoryUpdatePayload>("history-update-payload"),
};
