import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

const CLIENT_ID = 'aea2d8f6f68b44ab954129a2ea3f0862';
const AUTH_ENDPOINT = 'https://accounts.spotify.com/authorize';
const SCOPES = 'user-read-currently-playing user-read-playback-state user-modify-playback-state';

interface SpotifyTrack {
    name: string;
    artist: string;
    albumArt: string;
    duration_ms: number;
    progress_ms: number;
    is_playing: boolean;
}

interface SpotifyTokenResponse {
    access_token: string;
    token_type: string;
    scope: string;
    expires_in: number;
    refresh_token?: string;
}

export function useSpotify() {
    const [token, setToken] = useState<string | null>(localStorage.getItem('spotify_access_token'));
    const [track, setTrack] = useState<SpotifyTrack | null>(null);
    const [localProgress, setLocalProgress] = useState<number>(0);
    const [isAuthenticated, setIsAuthenticated] = useState<boolean>(!!token);

    const isHandlingCallback = useRef(false);
    const isSeeking = useRef(false);
    const seekTimeout = useRef<any>(null);

    const logout = useCallback(() => {
        localStorage.removeItem('spotify_access_token');
        localStorage.removeItem('spotify_refresh_token');
        localStorage.removeItem('spotify_code_verifier');
        localStorage.removeItem('spotify_redirect_uri');
        setToken(null);
        setTrack(null);
        setLocalProgress(0);
        setIsAuthenticated(false);
    }, []);

    // PKCE Utilities
    const generateRandomString = (length: number) => {
        const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        const values = crypto.getRandomValues(new Uint8Array(length));
        return values.reduce((acc, x) => acc + possible[x % possible.length], "");
    }

    const sha256 = async (plain: string) => {
        const encoder = new TextEncoder();
        const data = encoder.encode(plain);
        return window.crypto.subtle.digest('SHA-256', data);
    }

    const base64encode = (input: ArrayBuffer) => {
        return btoa(String.fromCharCode(...new Uint8Array(input)))
            .replace(/=/g, '')
            .replace(/\+/g, '-')
            .replace(/\//g, '_');
    }

    const login = async () => {
        const codeVerifier = generateRandomString(64);
        const hashed = await sha256(codeVerifier);
        const codeChallenge = base64encode(hashed);

        const redirectUri = window.location.origin.replace('localhost', '127.0.0.1') + window.location.pathname;
        window.localStorage.setItem('spotify_code_verifier', codeVerifier);
        window.localStorage.setItem('spotify_redirect_uri', redirectUri);

        const params = new URLSearchParams({
            response_type: 'code',
            client_id: CLIENT_ID,
            scope: SCOPES,
            code_challenge_method: 'S256',
            code_challenge: codeChallenge,
            redirect_uri: redirectUri,
        });

        window.location.href = `${AUTH_ENDPOINT}?${params.toString()}`;
    }

    const handleCallback = useCallback(async (code: string) => {
        if (isHandlingCallback.current) return;
        isHandlingCallback.current = true;

        const codeVerifier = window.localStorage.getItem('spotify_code_verifier');
        const redirectUri = window.localStorage.getItem('spotify_redirect_uri');

        try {
            const data = await invoke<SpotifyTokenResponse>('backend_spotify_token_exchange', {
                clientId: CLIENT_ID,
                code,
                redirectUri: redirectUri!,
                codeVerifier: codeVerifier!,
            });

            if (data.access_token) {
                localStorage.setItem('spotify_access_token', data.access_token);
                if (data.refresh_token) {
                    localStorage.setItem('spotify_refresh_token', data.refresh_token);
                }
                setToken(data.access_token);
                setIsAuthenticated(true);
                window.history.replaceState({}, document.title, window.location.pathname);
            }
        } catch (error) {
            console.error('Spotify token exchange failed:', error);
            logout();
            alert(`Spotify Auth Error: ${error}\n\nPlease try connecting again.`);
            window.history.replaceState({}, document.title, window.location.pathname);
        } finally {
            isHandlingCallback.current = false;
        }
    }, [logout]);

    const refreshToken = useCallback(async () => {
        const refresh_token = localStorage.getItem('spotify_refresh_token');
        if (!refresh_token) return null;

        try {
            const data = await invoke<SpotifyTokenResponse>('backend_spotify_refresh_token', {
                clientId: CLIENT_ID,
                refreshToken: refresh_token,
            });

            if (data.access_token) {
                localStorage.setItem('spotify_access_token', data.access_token);
                setToken(data.access_token);
                return data.access_token;
            }
        } catch (error) {
            console.error('Spotify token refresh failed:', error);
            if (error && typeof error === 'string' && error.includes('invalid_grant')) {
                logout();
            }
        }
        return null;
    }, [logout]);

    const fetchCurrentTrack = useCallback(async (currentToken?: string) => {
        const activeToken = currentToken || token;
        if (!activeToken) return;

        try {
            const response = await fetch('https://api.spotify.com/v1/me/player/currently-playing', {
                headers: { Authorization: `Bearer ${activeToken}` },
            });

            if (response.status === 401) {
                const newToken = await refreshToken();
                if (newToken) fetchCurrentTrack(newToken);
                return;
            }

            if (response.status === 204) {
                setTrack(null);
                setLocalProgress(0);
                return;
            }

            if (!response.ok) {
                setTrack(null);
                return;
            }

            const data = await response.json();
            if (data && data.item) {
                const newTrack = {
                    name: data.item.name,
                    artist: data.item.artists.map((a: any) => a.name).join(', '),
                    albumArt: data.item.album.images[0]?.url,
                    duration_ms: data.item.duration_ms,
                    progress_ms: data.progress_ms,
                    is_playing: data.is_playing,
                };
                setTrack(newTrack);

                // Only sync local progress with API if we aren't currently seeking
                if (!isSeeking.current) {
                    setLocalProgress(data.progress_ms);
                }
            } else {
                setTrack(null);
                setLocalProgress(0);
            }
        } catch (error) {
            console.error('Error fetching Spotify track:', error);
        }
    }, [token, refreshToken]);

    // Local interpolator for smooth progress
    useEffect(() => {
        if (!track?.is_playing || isSeeking.current) return;

        const interval = setInterval(() => {
            setLocalProgress(prev => {
                const next = prev + 100;
                return next > track.duration_ms ? track.duration_ms : next;
            });
        }, 100);

        return () => clearInterval(interval);
    }, [track?.is_playing, track?.duration_ms]);

    const spotifyFetch = async (endpoint: string, method: string = 'GET', body?: any) => {
        if (!token) return;
        const url = `https://api.spotify.com/v1/me/player/${endpoint}`;

        let response = await fetch(url, {
            method,
            headers: {
                Authorization: `Bearer ${token}`,
                'Content-Type': 'application/json',
            },
            body: body ? JSON.stringify(body) : undefined,
        });

        if (response.status === 401) {
            const newToken = await refreshToken();
            if (newToken) {
                response = await fetch(url, {
                    method,
                    headers: {
                        Authorization: `Bearer ${newToken}`,
                        'Content-Type': 'application/json',
                    },
                    body: body ? JSON.stringify(body) : undefined,
                });
            }
        }

        if (!response.ok && response.status !== 204) {
            try {
                const err = await response.json();
                console.error(`Spotify ${method} ${endpoint} failed:`, err);
                if (response.status === 403) {
                    alert("Spotify Premium is required for playback controls.");
                }
            } catch (e) {
                console.error(`Spotify ${method} ${endpoint} failed with status ${response.status}`);
            }
        }

        // Use a slightly longer delay after seeking to let Spotify's internal state catch up
        const delay = endpoint.startsWith('seek') ? 1500 : 500;
        setTimeout(() => fetchCurrentTrack(), delay);
    };

    const togglePlayback = () => {
        const endpoint = track?.is_playing ? 'pause' : 'play';
        spotifyFetch(endpoint, 'PUT');
        if (track) setTrack({ ...track, is_playing: !track.is_playing });
    };

    const skipNext = () => spotifyFetch('next', 'POST');
    const skipPrevious = () => spotifyFetch('previous', 'POST');

    const seek = (position_ms: number) => {
        // Optimistic update
        setLocalProgress(position_ms);

        // Stabilize: Lock API sync for 2 seconds to prevent "jumping back"
        isSeeking.current = true;
        if (seekTimeout.current) clearTimeout(seekTimeout.current);

        seekTimeout.current = setTimeout(() => {
            isSeeking.current = false;
        }, 2000);

        spotifyFetch(`seek?position_ms=${position_ms}`, 'PUT');
    };

    useEffect(() => {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get('code');
        const error = urlParams.get('error');

        if (error) {
            console.error('Spotify Auth Query Error:', error);
            logout();
            window.history.replaceState({}, document.title, window.location.pathname);
        } else if (code) {
            handleCallback(code);
        }
    }, [handleCallback, logout]);

    useEffect(() => {
        if (isAuthenticated) {
            fetchCurrentTrack();
            const interval = setInterval(() => fetchCurrentTrack(), 3000);
            return () => clearInterval(interval);
        }
    }, [isAuthenticated, fetchCurrentTrack]);

    return {
        track,
        progress: localProgress, // Export the smooth local progress
        login,
        logout,
        isAuthenticated,
        togglePlayback,
        skipNext,
        skipPrevious,
        seek
    };
}
