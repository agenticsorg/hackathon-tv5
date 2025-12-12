'use client';

import { useEffect, useState } from 'react';
import { MapContainer, GeoJSON, useMap } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';
import L from 'leaflet';
import { ISO2_TO_ISO3, ISO3_TO_ISO2 } from '@/lib/country-codes';

// Fix for default marker icon in Leaflet with Next.js
// (Though we are using GeoJSON, it's good practice to fix this global issue)
delete (L.Icon.Default.prototype as any)._getIconUrl;
L.Icon.Default.mergeOptions({
    iconRetinaUrl: '',
    iconUrl: '',
    shadowUrl: '',
});

interface LeafletMapProps {
    onSelectCountry: (code: string) => void;
    selectedCountry: string;
}

// Helper to fit bounds
function MapController() {
    const map = useMap();
    useEffect(() => {
        map.invalidateSize();
    }, [map]);
    return null;
}

export default function LeafletMap({ onSelectCountry, selectedCountry }: LeafletMapProps) {
    const [geoJsonData, setGeoJsonData] = useState<any>(null);

    useEffect(() => {
        fetch('/world-countries.json')
            .then((res) => res.json())
            .then((data) => setGeoJsonData(data));
    }, []);

    const style = (feature: any) => {
        // Convert selected 2-letter code to 3-letter for matching with GeoJSON ID
        const selectedCountry3 = ISO2_TO_ISO3[selectedCountry] || selectedCountry;
        const isSelected = feature.id === selectedCountry3;

        return {
            fillColor: isSelected ? '#aa55f5' : '#2a2a2a',
            weight: 1,
            opacity: 1,
            color: '#5b298aff', // Gold/Brown outline
            dashArray: '',
            fillOpacity: isSelected ? 0.9 : 0.7,
        };
    };

    const onEachFeature = (feature: any, layer: L.Layer) => {
        // Convert selected 2-letter code to 3-letter for comparison
        const selectedCountry3 = ISO2_TO_ISO3[selectedCountry] || selectedCountry;

        layer.on({
            mouseover: (e) => {
                const layer = e.target;
                // Don't change style if it's the selected country (keep it highlighted)
                if (feature.id !== selectedCountry3) {
                    layer.setStyle({
                        fillColor: '#aa55f5',
                        fillOpacity: 0.5,
                    });
                }
            },
            mouseout: (e) => {
                const layer = e.target;
                if (feature.id !== selectedCountry3) {
                    layer.setStyle({
                        fillColor: '#2a2a2a',
                        fillOpacity: 0.7,
                    });
                } else {
                    // Ensure selected country stays highlighted
                    layer.setStyle({
                        fillColor: '#aa55f5',
                        fillOpacity: 0.9,
                    });
                }
            },
            click: () => {
                // Convert 3-letter ID from GeoJSON to 2-letter code for the app
                const code2 = ISO3_TO_ISO2[feature.id] || feature.id;
                onSelectCountry(code2);
            },
        });
    };

    if (!geoJsonData) {
        return <div className="w-full h-full flex items-center justify-center text-gray-500">Loading Map...</div>;
    }

    return (
        <MapContainer
            center={[20, 0]}
            zoom={2}
            style={{ height: '100%', width: '100%', background: '#0f0f0f' }}
            zoomControl={false}
            attributionControl={false}
            maxBounds={[[-90, -180], [90, 180]]}
            maxBoundsViscosity={1.0}
            minZoom={2}
        >
            <MapController />
            <GeoJSON
                data={geoJsonData}
                style={style}
                onEachFeature={onEachFeature}
                // Force re-render when selectedCountry changes to update styles
                key={selectedCountry}
            />
        </MapContainer>
    );
}
