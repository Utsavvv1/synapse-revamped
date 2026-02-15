import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

const CLIENT_ID = 'aea2d8f6f68b44ab954129a2ea3f0862';
const AUTH_ENDPOINT = 'https://accounts.spotify.com/authorize';
const SCOPES = 'user-read-currently-playing user-read-playback-state';

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
    const [isAuthenticated, setIsAuthenticated] = useState<boolean>(!!token);
    const isHandlingCallback = useRef(false);

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

        console.log('--- Spotify Auth Debug ---');
        console.log('window.location.origin:', window.location.origin);
        console.log('window.location.pathname:', window.location.pathname);
        console.log('Redirect URI being sent:', redirectUri);

        const params = new URLSearchParams({
            response_type: 'code',
            client_id: CLIENT_ID,
            scope: SCOPES,
            code_challenge_method: 'S256',
            code_challenge: codeChallenge,
            redirect_uri: redirectUri,
        });

        const authUrl = `${AUTH_ENDPOINT}?${params.toString()}`;
        console.log('Full Auth URL:', authUrl);
        console.log('---------------------------');

        window.location.href = authUrl;
    }

    const handleCallback = useCallback(async (code: string) => {
        if (isHandlingCallback.current) return;
        isHandlingCallback.current = true;

        const codeVerifier = window.localStorage.getItem('spotify_code_verifier');
        const redirectUri = window.localStorage.getItem('spotify_redirect_uri');

        console.log('Exchanging code for token via Rust proxy...');
        console.log('Using Code:', code);
        console.log('Using Redirect URI:', redirectUri);

        try {
            const data = await invoke<SpotifyTokenResponse>('backend_spotify_token_exchange', {
                clientId: CLIENT_ID,
                code,
                redirectUri: redirectUri!,
                codeVerifier: codeVerifier!,
            });

            if (data.access_token) {
                console.log('Successfully received access token!');
                localStorage.setItem('spotify_access_token', data.access_token);
                if (data.refresh_token) {
                    localStorage.setItem('spotify_refresh_token', data.refresh_token);
                }
                setToken(data.access_token);
                setIsAuthenticated(true);
                // Clear URL params
                window.history.replaceState({}, document.title, window.location.pathname);
            }
        } catch (error) {
            console.error('Spotify token exchange failed:', error);
            alert(`Spotify Auth Error: ${error}`);
            window.history.replaceState({}, document.title, window.location.pathname);
        } finally {
            isHandlingCallback.current = false;
        }
    }, []);

    const refreshToken = useCallback(async () => {
        const refresh_token = localStorage.getItem('spotify_refresh_token');
        if (!refresh_token) return;

        console.log('Refreshing token via Rust proxy...');

        try {
            const data = await invoke<SpotifyTokenResponse>('backend_spotify_refresh_token', {
                clientId: CLIENT_ID,
                refreshToken: refresh_token,
            });

            if (data.access_token) {
                localStorage.setItem('spotify_access_token', data.access_token);
                setToken(data.access_token);
            }
        } catch (error) {
            console.error('Spotify token refresh failed:', error);
        }
    }, []);

    const fetchCurrentTrack = useCallback(async () => {
        if (!token) return;

        try {
            const response = await fetch('https://api.spotify.com/v1/me/player/currently-playing', {
                headers: { Authorization: `Bearer ${token}` },
            });

            if (response.status === 204 || response.status > 400) {
                if (response.status === 401) {
                    await refreshToken();
                } else if (response.status !== 204) {
                    setTrack(null);
                }
                return;
            }

            const data = await response.json();
            if (data && data.item) {
                setTrack({
                    name: data.item.name,
                    artist: data.item.artists.map((a: any) => a.name).join(', '),
                    albumArt: data.item.album.images[0]?.url,
                    duration_ms: data.item.duration_ms,
                    progress_ms: data.progress_ms,
                    is_playing: data.is_playing,
                });
            } else {
                setTrack(null);
            }
        } catch (error) {
            console.error('Error fetching Spotify track:', error);
        }
    }, [token, refreshToken]);

    useEffect(() => {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get('code');
        const error = urlParams.get('error');

        if (error) {
            console.error('Spotify Auth Query Error:', error);
            alert(`Spotify Error: ${error}`);
            window.history.replaceState({}, document.title, window.location.pathname);
        } else if (code) {
            handleCallback(code);
        }
    }, [handleCallback]);

    useEffect(() => {
        if (isAuthenticated) {
            fetchCurrentTrack();
            const interval = setInterval(fetchCurrentTrack, 5000);
            return () => clearInterval(interval);
        }
    }, [isAuthenticated, fetchCurrentTrack]);

    return { track, login, isAuthenticated };
}
