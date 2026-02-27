import { Room, RoomOptions } from 'livekit-client';

export function getLiveKitUrl(): string {
  return process.env.NEXT_PUBLIC_LIVEKIT_URL || '';
}

export async function connectToRoom(token: string, url: string): Promise<Room> {
  const roomOptions: RoomOptions = {
    adaptiveStream: true,
    dynacast: true,
  };

  const room = new Room(roomOptions);
  await room.connect(url, token);
  return room;
}

export function formatCallDuration(startedAt: string): string {
  const start = new Date(startedAt).getTime();
  const now = Date.now();
  const totalSeconds = Math.floor((now - start) / 1000);

  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
  }
  return `${minutes}:${String(seconds).padStart(2, '0')}`;
}
