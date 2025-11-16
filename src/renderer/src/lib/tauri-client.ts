import { invoke } from '@tauri-apps/api/core';
import { listen as tauriListen } from '@tauri-apps/api/event';

// Tauri client that mimics the tipc client interface
// This allows minimal changes to the rest of the codebase

type ProcedureAction<TInput = void, TOutput = void> = TInput extends void
  ? () => Promise<TOutput>
  : (input: TInput) => Promise<TOutput>;

interface Router {
  restartApp: ProcedureAction;
  getUpdateInfo: ProcedureAction<void, any>;
  quitAndInstall: ProcedureAction;
  checkForUpdatesAndDownload: ProcedureAction<void, any>;
  openMicrophoneInSystemPreferences: ProcedureAction;
  hidePanelWindow: ProcedureAction;
  showContextMenu: ProcedureAction<{ x: number; y: number; selectedText?: string }>;
  getMicrophoneStatus: ProcedureAction<void, string>;
  isAccessibilityGranted: ProcedureAction<void, boolean>;
  requestAccesssbilityAccess: ProcedureAction<void, boolean>;
  requestMicrophoneAccess: ProcedureAction<void, boolean>;
  showPanelWindow: ProcedureAction;
  displayError: ProcedureAction<{ title?: string; message: string }>;
  createRecording: ProcedureAction<{
    recording: ArrayBuffer;
    duration: number;
    useFusion?: boolean;
  }, { transcript: string; fusionResult: any }>;
  getRecordingHistory: ProcedureAction<void, any[]>;
  deleteRecordingItem: ProcedureAction<{ id: string }>;
  deleteRecordingHistory: ProcedureAction;
  resizeStatusBarWindow: ProcedureAction<{ width: number; height: number; expanded?: boolean }>;
  toggleRecordingTranscript: ProcedureAction<{ id: string }, any>;
  getConfig: ProcedureAction<void, any>;
  saveConfig: ProcedureAction<{ config: any }>;
  recordEvent: ProcedureAction<{ type: 'start' | 'end' }>;
  initVoiceActivation: ProcedureAction<void, any>;
  startVoiceActivation: ProcedureAction;
  stopVoiceActivation: ProcedureAction;
  getVoiceActivationStatus: ProcedureAction<void, any>;
  cleanupVoiceActivation: ProcedureAction;
  initStreamingDictation: ProcedureAction<void, any>;
  startStreamingDictation: ProcedureAction;
  stopStreamingDictation: ProcedureAction;
  pauseStreamingDictation: ProcedureAction;
  resumeStreamingDictation: ProcedureAction;
  toggleStreamingDictation: ProcedureAction;
  getStreamingDictationStatus: ProcedureAction<void, any>;
  cleanupStreamingDictation: ProcedureAction;
  getActiveApplication: ProcedureAction<void, any>;
  updateActiveApplication: ProcedureAction;
  getEffectiveConfig: ProcedureAction<void, any>;
  createAppRule: ProcedureAction<{ rule: any }, any>;
  updateAppRule: ProcedureAction<{ rule: any }, any>;
  deleteAppRule: ProcedureAction<{ id: string }>;
  getAppRules: ProcedureAction<void, any[]>;
  testAppRule: ProcedureAction<{ rule: any }, boolean>;
  getRecordingState: ProcedureAction<void, any>;
  getProfiles: ProcedureAction<void, any[]>;
  getActiveProfileId: ProcedureAction<void, string>;
  createProfile: ProcedureAction<{ name: string; description?: string; baseConfig?: any }, any>;
  updateProfile: ProcedureAction<{ profileId: string; updates: any }>;
  deleteProfile: ProcedureAction<{ profileId: string }, boolean>;
  switchProfile: ProcedureAction<{ profileId: string }, boolean>;
  duplicateProfile: ProcedureAction<{ profileId: string; newName: string }, any>;
  testFusionConfiguration: ProcedureAction<void, any>;
  getFusionConfig: ProcedureAction<void, any>;
  updateFusionConfig: ProcedureAction<{ fusionConfig: any }>;
  testContextDetection: ProcedureAction<void, any>;
  getCurrentAppInfo: ProcedureAction<void, any>;
  detectContextForApp: ProcedureAction<{ appInfo: any }, string>;
  getEffectiveFormattingConfig: ProcedureAction<void, any>;
  previewContextFormatting: ProcedureAction<{ transcript: string; formattingConfig: any }, any>;
}

// Create a proxy that converts tipc-style calls to Tauri invoke calls
function createTauriClient(): Router {
  const handler = {
    get(_target: any, prop: string) {
      return async (input?: any) => {
        try {
          // Convert camelCase to snake_case for Tauri commands
          const commandName = toSnakeCase(prop);

          // For commands with input, unwrap and pass parameters directly
          if (input !== undefined) {
            // Handle special cases
            if (prop === 'createRecording' && input.recording) {
              // Convert ArrayBuffer to array for Tauri
              const recordingArray = Array.from(new Uint8Array(input.recording));
              return await invoke(commandName, {
                recording: recordingArray,
                duration: input.duration,
                useFusion: input.useFusion,
              });
            } else if (prop === 'saveConfig' && input.config) {
              // Unwrap config parameter
              return await invoke(commandName, { config: input.config });
            } else {
              // Pass input directly for other commands
              return await invoke(commandName, input);
            }
          } else {
            return await invoke(commandName);
          }
        } catch (error) {
          console.error(`Error invoking ${prop}:`, error);
          throw error;
        }
      };
    },
  };

  return new Proxy({} as Router, handler);
}

function toSnakeCase(str: string): string {
  return str.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`).replace(/^_/, '');
}

export const tipcClient = createTauriClient();

// Renderer handlers for events from backend to frontend
interface RendererHandlers {
  refreshRecordingHistory: {
    send: () => void;
  };
}

export const rendererHandlers: RendererHandlers = {
  refreshRecordingHistory: {
    send: () => {
      // Emit event that components can listen to
      window.dispatchEvent(new CustomEvent('refresh-recording-history'));
    },
  },
};

// Helper to listen to backend events
export async function listenToBackendEvent(
  eventName: string,
  callback: (data: any) => void
) {
  return await tauriListen(eventName, (event) => {
    callback(event.payload);
  });
}
