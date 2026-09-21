export interface Insets {
  top: number;
  bottom: number;
  left: number;
  right: number;
}

export interface CameraCutout {
  type: string;
  x: number;
  y: number;
  radius: number;
}

export interface DeviceProfile {
  name: string;
  id: string;
  platform: 'android' | 'ios';
  width: number;
  height: number;
  logicalWidth: number;
  logicalHeight: number;
  density: number;
  fontScale: number;
  safeArea: Insets;
  cornerRadius: number;
  cameraCutout?: CameraCutout | null;
}
