'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import { Mic, MicOff, Video, VideoOff, Monitor, PhoneOff, Minimize2, Maximize2 } from 'lucide-react';
import { Room, RemoteTrack, Track, RoomEvent } from 'livekit-client';
import { useWebSocketStore } from '@/lib/websocket';
import { apiClient } from '@/lib/api';
import { connectToRoom, formatCallDuration } from '@/lib/livekit';

interface ParticipantTile {
  identity: string;
  name: string;
  videoTrack: RemoteTrack | null;
  audioTrack: RemoteTrack | null;
  isSpeaking: boolean;
  isMuted: boolean;
  isLocal: boolean;
}

export default function CallOverlay() {
  const { currentCall, setCurrentCall, activeChannelId, activeDmId } = useWebSocketStore();
  const roomRef = useRef<Room | null>(null);
  const [participants, setParticipants] = useState<ParticipantTile[]>([]);
  const [isMuted, setIsMuted] = useState(false);
  const [isCameraOff, setIsCameraOff] = useState(true);
  const [isScreenSharing, setIsScreenSharing] = useState(false);
  const [isPip, setIsPip] = useState(false);
  const [duration, setDuration] = useState('0:00');
  const videoRefs = useRef<Record<string, HTMLVideoElement | null>>({});
  const audioRefs = useRef<Record<string, HTMLAudioElement | null>>({});

  const isVideo = currentCall?.call_type === 'video';

  // Check PiP: if user navigated away from the call's channel/DM
  useEffect(() => {
    if (!currentCall) return;
    const isInCallContext =
      (currentCall.channel_id && activeChannelId === currentCall.channel_id) ||
      (currentCall.dm_id && activeDmId === currentCall.dm_id);
    setIsPip(!isInCallContext);
  }, [activeChannelId, activeDmId, currentCall]);

  // Duration timer
  useEffect(() => {
    if (!currentCall) return;
    const interval = setInterval(() => {
      setDuration(formatCallDuration(currentCall.started_at));
    }, 1000);
    return () => clearInterval(interval);
  }, [currentCall]);

  const updateParticipants = useCallback((room: Room) => {
    const tiles: ParticipantTile[] = [];

    // Local participant
    const local = room.localParticipant;
    tiles.push({
      identity: local.identity,
      name: local.name || local.identity,
      videoTrack: null,
      audioTrack: null,
      isSpeaking: local.isSpeaking,
      isMuted: !local.isMicrophoneEnabled,
      isLocal: true,
    });

    // Remote participants
    for (const participant of room.remoteParticipants.values()) {
      let videoTrack: RemoteTrack | null = null;
      let audioTrack: RemoteTrack | null = null;

      for (const pub of participant.trackPublications.values()) {
        if (pub.track && pub.kind === Track.Kind.Video && pub.source === Track.Source.Camera) {
          videoTrack = pub.track as RemoteTrack;
        }
        if (pub.track && pub.kind === Track.Kind.Audio) {
          audioTrack = pub.track as RemoteTrack;
        }
      }

      tiles.push({
        identity: participant.identity,
        name: participant.name || participant.identity,
        videoTrack,
        audioTrack,
        isSpeaking: participant.isSpeaking,
        isMuted: !participant.isMicrophoneEnabled,
        isLocal: false,
      });
    }

    setParticipants(tiles);
  }, []);

  // Connect to room
  useEffect(() => {
    if (!currentCall) return;

    let room: Room | null = null;

    const connect = async () => {
      try {
        room = await connectToRoom(currentCall.token, currentCall.livekit_url);
        roomRef.current = room;

        // Enable camera for video calls
        if (isVideo) {
          await room.localParticipant.setCameraEnabled(true);
          setIsCameraOff(false);
        }

        const onUpdate = () => updateParticipants(room!);

        room.on(RoomEvent.TrackSubscribed, onUpdate);
        room.on(RoomEvent.TrackUnsubscribed, onUpdate);
        room.on(RoomEvent.ParticipantConnected, onUpdate);
        room.on(RoomEvent.ParticipantDisconnected, onUpdate);
        room.on(RoomEvent.ActiveSpeakersChanged, onUpdate);
        room.on(RoomEvent.TrackMuted, onUpdate);
        room.on(RoomEvent.TrackUnmuted, onUpdate);

        updateParticipants(room);
      } catch (err) {
        console.error('Failed to connect to LiveKit room:', err);
      }
    };

    connect();

    return () => {
      if (room) {
        room.disconnect();
        roomRef.current = null;
      }
    };
  }, [currentCall, isVideo, updateParticipants]);

  // Attach video/audio tracks
  useEffect(() => {
    for (const p of participants) {
      if (p.videoTrack) {
        const el = videoRefs.current[p.identity];
        if (el) {
          p.videoTrack.attach(el);
        }
      }
      if (p.audioTrack && !p.isLocal) {
        const el = audioRefs.current[p.identity];
        if (el) {
          p.audioTrack.attach(el);
        }
      }
    }
  }, [participants]);

  if (!currentCall) return null;

  const handleToggleMute = async () => {
    const room = roomRef.current;
    if (!room) return;
    await room.localParticipant.setMicrophoneEnabled(isMuted);
    setIsMuted(!isMuted);
  };

  const handleToggleCamera = async () => {
    const room = roomRef.current;
    if (!room) return;
    await room.localParticipant.setCameraEnabled(isCameraOff);
    setIsCameraOff(!isCameraOff);
  };

  const handleToggleScreenShare = async () => {
    const room = roomRef.current;
    if (!room) return;
    await room.localParticipant.setScreenShareEnabled(!isScreenSharing);
    setIsScreenSharing(!isScreenSharing);
  };

  const handleEndCall = async () => {
    try {
      await apiClient.leaveCall(currentCall.call_id);
    } catch {
      // Ignore — may already be ended
    }
    roomRef.current?.disconnect();
    roomRef.current = null;
    setCurrentCall(null);
  };

  // PiP mode — small floating window
  if (isPip) {
    return (
      <div className="animate-pip-in fixed bottom-4 right-4 z-[55] w-72 overflow-hidden rounded-lg bg-gray-900 shadow-2xl ring-1 ring-gray-700">
        <div className="flex items-center justify-between px-3 py-2">
          <span className="text-xs text-gray-400">{duration}</span>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setIsPip(false)}
              className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
              aria-label="Expand call"
            >
              <Maximize2 className="h-3.5 w-3.5" />
            </button>
          </div>
        </div>
        <div className="flex items-center gap-2 border-t border-gray-700 px-3 py-2">
          <button
            onClick={handleToggleMute}
            className={`rounded p-1.5 ${isMuted ? 'bg-red-600 text-white' : 'text-gray-400 hover:bg-gray-800 hover:text-white'}`}
            aria-label={isMuted ? 'Unmute' : 'Mute'}
          >
            {isMuted ? <MicOff className="h-4 w-4" /> : <Mic className="h-4 w-4" />}
          </button>
          {isVideo && (
            <button
              onClick={handleToggleCamera}
              className={`rounded p-1.5 ${isCameraOff ? 'bg-red-600 text-white' : 'text-gray-400 hover:bg-gray-800 hover:text-white'}`}
              aria-label={isCameraOff ? 'Turn camera on' : 'Turn camera off'}
            >
              {isCameraOff ? <VideoOff className="h-4 w-4" /> : <Video className="h-4 w-4" />}
            </button>
          )}
          <button
            onClick={handleEndCall}
            className="ml-auto rounded bg-red-600 p-1.5 text-white hover:bg-red-700"
            aria-label="End call"
          >
            <PhoneOff className="h-4 w-4" />
          </button>
        </div>
      </div>
    );
  }

  // Full overlay
  const gridCols =
    participants.length <= 2 ? 'grid-cols-1' : participants.length <= 4 ? 'grid-cols-2' : 'grid-cols-3';

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-gray-950">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-gray-800 px-4 py-3">
        <span className="text-sm font-medium text-white">
          {isVideo ? 'Video' : 'Audio'} Call
        </span>
        <div className="flex items-center gap-3">
          <span className="text-sm text-gray-400">{duration}</span>
          <button
            onClick={() => setIsPip(true)}
            className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
            aria-label="Minimize to PiP"
          >
            <Minimize2 className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Participant grid */}
      <div className={`flex-1 grid ${gridCols} gap-2 p-4 auto-rows-fr`}>
        {participants.map((p) => (
          <div
            key={p.identity}
            className={`relative flex items-center justify-center overflow-hidden rounded-lg bg-gray-800 ${
              p.isSpeaking ? 'ring-2 ring-green-500' : ''
            }`}
          >
            {p.videoTrack ? (
              <video
                ref={(el) => { videoRefs.current[p.identity] = el; }}
                autoPlay
                playsInline
                muted={p.isLocal}
                className="h-full w-full object-cover"
              />
            ) : (
              <div className="flex flex-col items-center gap-2">
                <div className="flex h-16 w-16 items-center justify-center rounded-full bg-gray-700 text-2xl font-bold text-white">
                  {p.name.charAt(0).toUpperCase()}
                </div>
                <span className="text-sm text-gray-300">{p.name}</span>
              </div>
            )}
            {/* Audio element for remote participants */}
            {!p.isLocal && p.audioTrack && (
              <audio
                ref={(el) => { audioRefs.current[p.identity] = el; }}
                autoPlay
              />
            )}
            {/* Name overlay */}
            <div className="absolute bottom-2 left-2 flex items-center gap-1 rounded bg-black/60 px-2 py-0.5">
              <span className="text-xs text-white">{p.isLocal ? 'You' : p.name}</span>
              {p.isMuted && <MicOff className="h-3 w-3 text-red-400" />}
            </div>
          </div>
        ))}
      </div>

      {/* Controls bar */}
      <div className="flex items-center justify-center gap-3 border-t border-gray-800 px-4 py-4">
        <button
          onClick={handleToggleMute}
          className={`rounded-full p-3 ${isMuted ? 'bg-red-600 text-white' : 'bg-gray-700 text-white hover:bg-gray-600'}`}
          aria-label={isMuted ? 'Unmute' : 'Mute'}
        >
          {isMuted ? <MicOff className="h-5 w-5" /> : <Mic className="h-5 w-5" />}
        </button>

        {isVideo && (
          <button
            onClick={handleToggleCamera}
            className={`rounded-full p-3 ${isCameraOff ? 'bg-red-600 text-white' : 'bg-gray-700 text-white hover:bg-gray-600'}`}
            aria-label={isCameraOff ? 'Turn camera on' : 'Turn camera off'}
          >
            {isCameraOff ? <VideoOff className="h-5 w-5" /> : <Video className="h-5 w-5" />}
          </button>
        )}

        <button
          onClick={handleToggleScreenShare}
          className={`rounded-full p-3 ${isScreenSharing ? 'bg-blue-600 text-white' : 'bg-gray-700 text-white hover:bg-gray-600'}`}
          aria-label={isScreenSharing ? 'Stop screen share' : 'Share screen'}
        >
          <Monitor className="h-5 w-5" />
        </button>

        <button
          onClick={handleEndCall}
          className="rounded-full bg-red-600 p-3 text-white hover:bg-red-700"
          aria-label="End call"
        >
          <PhoneOff className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}
