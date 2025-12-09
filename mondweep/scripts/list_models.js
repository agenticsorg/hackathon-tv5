
const https = require('https');

// Use the key provided by the user
const API_KEY = "AIzaSyA0Cb_VeanorrQyB1e1hQ_5A3TdbVJrI34";

const options = {
    hostname: 'generativelanguage.googleapis.com',
    path: `/v1beta/models?key=${API_KEY}`,
    method: 'GET',
};

const req = https.request(options, (res) => {
    let data = '';

    res.on('data', (chunk) => {
        data += chunk;
    });

    res.on('end', () => {
        if (res.statusCode === 200) {
            const response = JSON.parse(data);
            console.log("Available Models:");
            if (response.models) {
                response.models.forEach(m => {
                    if (m.supportedGenerationMethods.includes('generateContent')) {
                        console.log(`- ${m.name} (Version: ${m.version})`);
                    }
                });
            } else {
                console.log("No models found?", response);
            }
        } else {
            console.error(`Error: ${res.statusCode}`);
            console.error(data);
        }
    });

});

req.on('error', (error) => {
    console.error(error);
});

req.end();
