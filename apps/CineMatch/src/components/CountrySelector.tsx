'use client';

import React, { useState, useMemo } from 'react';
import { Map as MapIcon, X } from 'lucide-react';
import { useMatchStore } from '@/store/match-store';
import dynamic from 'next/dynamic';

// Dynamically import LeafletMap to avoid SSR issues
const LeafletMap = dynamic(() => import('./LeafletMap'), {
    ssr: false,
    loading: () => <div className="w-full h-full flex items-center justify-center text-gray-500 bg-[#121212]">Loading Map...</div>
});

// Full ISO 3166-1 alpha-2 Country List
const COUNTRIES = [
    { "name": "Afghanistan", "code": "AF" }, { "name": "Åland Islands", "code": "AX" }, { "name": "Albania", "code": "AL" }, { "name": "Algeria", "code": "DZ" }, { "name": "American Samoa", "code": "AS" }, { "name": "Andorra", "code": "AD" }, { "name": "Angola", "code": "AO" }, { "name": "Anguilla", "code": "AI" }, { "name": "Antarctica", "code": "AQ" }, { "name": "Antigua and Barbuda", "code": "AG" }, { "name": "Argentina", "code": "AR" }, { "name": "Armenia", "code": "AM" }, { "name": "Aruba", "code": "AW" }, { "name": "Australia", "code": "AU" }, { "name": "Austria", "code": "AT" }, { "name": "Azerbaijan", "code": "AZ" }, { "name": "Bahamas", "code": "BS" }, { "name": "Bahrain", "code": "BH" }, { "name": "Bangladesh", "code": "BD" }, { "name": "Barbados", "code": "BB" }, { "name": "Belarus", "code": "BY" }, { "name": "Belgium", "code": "BE" }, { "name": "Belize", "code": "BZ" }, { "name": "Benin", "code": "BJ" }, { "name": "Bermuda", "code": "BM" }, { "name": "Bhutan", "code": "BT" }, { "name": "Bolivia", "code": "BO" }, { "name": "Bonaire, Sint Eustatius and Saba", "code": "BQ" }, { "name": "Bosnia and Herzegovina", "code": "BA" }, { "name": "Botswana", "code": "BW" }, { "name": "Bouvet Island", "code": "BV" }, { "name": "Brazil", "code": "BR" }, { "name": "British Indian Ocean Territory", "code": "IO" }, { "name": "Brunei Darussalam", "code": "BN" }, { "name": "Bulgaria", "code": "BG" }, { "name": "Burkina Faso", "code": "BF" }, { "name": "Burundi", "code": "BI" }, { "name": "Cabo Verde", "code": "CV" }, { "name": "Cambodia", "code": "KH" }, { "name": "Cameroon", "code": "CM" }, { "name": "Canada", "code": "CA" }, { "name": "Cayman Islands", "code": "KY" }, { "name": "Central African Republic", "code": "CF" }, { "name": "Chad", "code": "TD" }, { "name": "Chile", "code": "CL" }, { "name": "China", "code": "CN" }, { "name": "Christmas Island", "code": "CX" }, { "name": "Cocos (Keeling) Islands", "code": "CC" }, { "name": "Colombia", "code": "CO" }, { "name": "Comoros", "code": "KM" }, { "name": "Congo", "code": "CG" }, { "name": "Congo, Democratic Republic of the", "code": "CD" }, { "name": "Cook Islands", "code": "CK" }, { "name": "Costa Rica", "code": "CR" }, { "name": "Côte d'Ivoire", "code": "CI" }, { "name": "Croatia", "code": "HR" }, { "name": "Cuba", "code": "CU" }, { "name": "Curaçao", "code": "CW" }, { "name": "Cyprus", "code": "CY" }, { "name": "Czechia", "code": "CZ" }, { "name": "Denmark", "code": "DK" }, { "name": "Djibouti", "code": "DJ" }, { "name": "Dominica", "code": "DM" }, { "name": "Dominican Republic", "code": "DO" }, { "name": "Ecuador", "code": "EC" }, { "name": "Egypt", "code": "EG" }, { "name": "El Salvador", "code": "SV" }, { "name": "Equatorial Guinea", "code": "GQ" }, { "name": "Eritrea", "code": "ER" }, { "name": "Estonia", "code": "EE" }, { "name": "Eswatini", "code": "SZ" }, { "name": "Ethiopia", "code": "ET" }, { "name": "Falkland Islands (Malvinas)", "code": "FK" }, { "name": "Faroe Islands", "code": "FO" }, { "name": "Fiji", "code": "FJ" }, { "name": "Finland", "code": "FI" }, { "name": "France", "code": "FR" }, { "name": "French Guiana", "code": "GF" }, { "name": "French Polynesia", "code": "PF" }, { "name": "French Southern Territories", "code": "TF" }, { "name": "Gabon", "code": "GA" }, { "name": "Gambia", "code": "GM" }, { "name": "Georgia", "code": "GE" }, { "name": "Germany", "code": "DE" }, { "name": "Ghana", "code": "GH" }, { "name": "Gibraltar", "code": "GI" }, { "name": "Greece", "code": "GR" }, { "name": "Greenland", "code": "GL" }, { "name": "Grenada", "code": "GD" }, { "name": "Guadeloupe", "code": "GP" }, { "name": "Guam", "code": "GU" }, { "name": "Guatemala", "code": "GT" }, { "name": "Guernsey", "code": "GG" }, { "name": "Guinea", "code": "GN" }, { "name": "Guinea-Bissau", "code": "GW" }, { "name": "Guyana", "code": "GY" }, { "name": "Haiti", "code": "HT" }, { "name": "Heard Island and McDonald Islands", "code": "HM" }, { "name": "Holy See", "code": "VA" }, { "name": "Honduras", "code": "HN" }, { "name": "Hong Kong", "code": "HK" }, { "name": "Hungary", "code": "HU" }, { "name": "Iceland", "code": "IS" }, { "name": "India", "code": "IN" }, { "name": "Indonesia", "code": "ID" }, { "name": "Iran", "code": "IR" }, { "name": "Iraq", "code": "IQ" }, { "name": "Ireland", "code": "IE" }, { "name": "Isle of Man", "code": "IM" }, { "name": "Israel", "code": "IL" }, { "name": "Italy", "code": "IT" }, { "name": "Jamaica", "code": "JM" }, { "name": "Japan", "code": "JP" }, { "name": "Jersey", "code": "JE" }, { "name": "Jordan", "code": "JO" }, { "name": "Kazakhstan", "code": "KZ" }, { "name": "Kenya", "code": "KE" }, { "name": "Kiribati", "code": "KI" }, { "name": "Korea, Democratic People's Republic of", "code": "KP" }, { "name": "Korea, Republic of", "code": "KR" }, { "name": "Kuwait", "code": "KW" }, { "name": "Kyrgyzstan", "code": "KG" }, { "name": "Lao People's Democratic Republic", "code": "LA" }, { "name": "Latvia", "code": "LV" }, { "name": "Lebanon", "code": "LB" }, { "name": "Lesotho", "code": "LS" }, { "name": "Liberia", "code": "LR" }, { "name": "Libya", "code": "LY" }, { "name": "Liechtenstein", "code": "LI" }, { "name": "Lithuania", "code": "LT" }, { "name": "Luxembourg", "code": "LU" }, { "name": "Macao", "code": "MO" }, { "name": "Madagascar", "code": "MG" }, { "name": "Malawi", "code": "MW" }, { "name": "Malaysia", "code": "MY" }, { "name": "Maldives", "code": "MV" }, { "name": "Mali", "code": "ML" }, { "name": "Malta", "code": "MT" }, { "name": "Marshall Islands", "code": "MH" }, { "name": "Martinique", "code": "MQ" }, { "name": "Mauritania", "code": "MR" }, { "name": "Mauritius", "code": "MU" }, { "name": "Mayotte", "code": "YT" }, { "name": "Mexico", "code": "MX" }, { "name": "Micronesia", "code": "FM" }, { "name": "Moldova", "code": "MD" }, { "name": "Monaco", "code": "MC" }, { "name": "Mongolia", "code": "MN" }, { "name": "Montenegro", "code": "ME" }, { "name": "Montserrat", "code": "MS" }, { "name": "Morocco", "code": "MA" }, { "name": "Mozambique", "code": "MZ" }, { "name": "Myanmar", "code": "MM" }, { "name": "Namibia", "code": "NA" }, { "name": "Nauru", "code": "NR" }, { "name": "Nepal", "code": "NP" }, { "name": "Netherlands", "code": "NL" }, { "name": "New Caledonia", "code": "NC" }, { "name": "New Zealand", "code": "NZ" }, { "name": "Nicaragua", "code": "NI" }, { "name": "Niger", "code": "NE" }, { "name": "Nigeria", "code": "NG" }, { "name": "Niue", "code": "NU" }, { "name": "Norfolk Island", "code": "NF" }, { "name": "North Macedonia", "code": "MK" }, { "name": "Northern Mariana Islands", "code": "MP" }, { "name": "Norway", "code": "NO" }, { "name": "Oman", "code": "OM" }, { "name": "Pakistan", "code": "PK" }, { "name": "Palau", "code": "PW" }, { "name": "Palestine, State of", "code": "PS" }, { "name": "Panama", "code": "PA" }, { "name": "Papua New Guinea", "code": "PG" }, { "name": "Paraguay", "code": "PY" }, { "name": "Peru", "code": "PE" }, { "name": "Philippines", "code": "PH" }, { "name": "Pitcairn", "code": "PN" }, { "name": "Poland", "code": "PL" }, { "name": "Portugal", "code": "PT" }, { "name": "Puerto Rico", "code": "PR" }, { "name": "Qatar", "code": "QA" }, { "name": "Réunion", "code": "RE" }, { "name": "Romania", "code": "RO" }, { "name": "Russian Federation", "code": "RU" }, { "name": "Rwanda", "code": "RW" }, { "name": "Saint Barthélemy", "code": "BL" }, { "name": "Saint Helena", "code": "SH" }, { "name": "Saint Kitts and Nevis", "code": "KN" }, { "name": "Saint Lucia", "code": "LC" }, { "name": "Saint Martin", "code": "MF" }, { "name": "Saint Pierre and Miquelon", "code": "PM" }, { "name": "Saint Vincent and the Grenadines", "code": "VC" }, { "name": "Samoa", "code": "WS" }, { "name": "San Marino", "code": "SM" }, { "name": "Sao Tome and Principe", "code": "ST" }, { "name": "Saudi Arabia", "code": "SA" }, { "name": "Senegal", "code": "SN" }, { "name": "Serbia", "code": "RS" }, { "name": "Seychelles", "code": "SC" }, { "name": "Sierra Leone", "code": "SL" }, { "name": "Singapore", "code": "SG" }, { "name": "Sint Maarten", "code": "SX" }, { "name": "Slovakia", "code": "SK" }, { "name": "Slovenia", "code": "SI" }, { "name": "Solomon Islands", "code": "SB" }, { "name": "Somalia", "code": "SO" }, { "name": "South Africa", "code": "ZA" }, { "name": "South Georgia", "code": "GS" }, { "name": "South Sudan", "code": "SS" }, { "name": "Spain", "code": "ES" }, { "name": "Sri Lanka", "code": "LK" }, { "name": "Sudan", "code": "SD" }, { "name": "Suriname", "code": "SR" }, { "name": "Svalbard and Jan Mayen", "code": "SJ" }, { "name": "Sweden", "code": "SE" }, { "name": "Switzerland", "code": "CH" }, { "name": "Syria", "code": "SY" }, { "name": "Taiwan", "code": "TW" }, { "name": "Tajikistan", "code": "TJ" }, { "name": "Tanzania", "code": "TZ" }, { "name": "Thailand", "code": "TH" }, { "name": "Timor-Leste", "code": "TL" }, { "name": "Togo", "code": "TG" }, { "name": "Tokelau", "code": "TK" }, { "name": "Tonga", "code": "TO" }, { "name": "Trinidad and Tobago", "code": "TT" }, { "name": "Tunisia", "code": "TN" }, { "name": "Turkey", "code": "TR" }, { "name": "Turkmenistan", "code": "TM" }, { "name": "Turks and Caicos Islands", "code": "TC" }, { "name": "Tuvalu", "code": "TV" }, { "name": "Uganda", "code": "UG" }, { "name": "Ukraine", "code": "UA" }, { "name": "United Arab Emirates", "code": "AE" }, { "name": "United Kingdom", "code": "GB" }, { "name": "United States", "code": "US" }, { "name": "Uruguay", "code": "UY" }, { "name": "Uzbekistan", "code": "UZ" }, { "name": "Vanuatu", "code": "VU" }, { "name": "Venezuela", "code": "VE" }, { "name": "Vietnam", "code": "VN" }, { "name": "Virgin Islands (British)", "code": "VG" }, { "name": "Virgin Islands (U.S.)", "code": "VI" }, { "name": "Wallis and Futuna", "code": "WF" }, { "name": "Western Sahara", "code": "EH" }, { "name": "Yemen", "code": "YE" }, { "name": "Zambia", "code": "ZM" }, { "name": "Zimbabwe", "code": "ZW" }
];

export function CountrySelector() {
    const [isOpen, setIsOpen] = useState(false);
    const { preferences, setPreferences } = useMatchStore();
    const currentCountry = preferences.userCountry || 'FR';

    const handleCountrySelect = (code: string) => {
        setPreferences({ userCountry: code });
    };

    // Calculate selected country for map (might need 3-letter code if we had reverse mapping, 
    // but for now we just pass the 2-letter code and let the map handle it if it can, 
    // or we accept that highlighting might be tricky without full reverse map.
    // Actually, let's just pass the current 2-letter code. 
    // The LeafletMap will need to match it against features. 
    // We'll update LeafletMap to handle this matching logic if needed, 
    // but for now let's pass the 2-letter code.

    return (
        <>
            <button
                onClick={() => setIsOpen(true)}
                className="p-3 rounded-full bg-white/10 hover:bg-white/20 transition-colors backdrop-blur-sm border border-white/10"
                aria-label="Select Country"
            >
                <MapIcon className="w-6 h-6 text-white" />
            </button>

            {isOpen && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
                    <div className="bg-[#1a1a1a] rounded-2xl border border-[#3a3a3a] w-full max-w-4xl p-6 relative shadow-2xl">
                        <button
                            onClick={() => setIsOpen(false)}
                            className="absolute top-4 right-4 p-2 rounded-full hover:bg-white/10 transition-colors z-10"
                        >
                            <X className="w-6 h-6 text-white" />
                        </button>

                        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-3">
                            <MapIcon className="w-6 h-6 text-[#aa55f5]" />
                            Select Your Country
                        </h2>

                        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                            {/* Map Section */}
                            <div className="lg:col-span-2 bg-[#121212] rounded-xl p-4 border border-[#3a3a3a] relative overflow-hidden h-[400px]">
                                <LeafletMap
                                    onSelectCountry={handleCountrySelect}
                                    selectedCountry={currentCountry}
                                />
                            </div>

                            {/* Controls Section */}
                            <div className="flex flex-col gap-6">
                                <div>
                                    <label className="block text-sm font-medium text-gray-400 mb-2">
                                        Current Selection
                                    </label>
                                    <div className="text-3xl font-bold text-white mb-1">
                                        {COUNTRIES.find(c => c.code === currentCountry)?.name || currentCountry}
                                    </div>
                                    <div className="text-sm text-[#aa55f5]">
                                        {currentCountry === 'FR' ? 'Default' : 'Custom Selection'}
                                    </div>
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-gray-400 mb-2">
                                        Choose from list
                                    </label>
                                    <select
                                        value={currentCountry}
                                        onChange={(e) => handleCountrySelect(e.target.value)}
                                        className="w-full bg-[#121212] border border-[#3a3a3a] rounded-lg px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-[#aa55f5] max-h-60 overflow-y-auto"
                                    >
                                        {COUNTRIES.map((country) => (
                                            <option key={country.code} value={country.code}>
                                                {country.name}
                                            </option>
                                        ))}
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
