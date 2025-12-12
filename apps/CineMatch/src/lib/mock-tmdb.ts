import type { Movie, TVShow, MediaContent } from '@/types/media';

export const MOCK_MOVIES: Movie[] = [
  {
    id: 1,
    title: "Inception",
    overview: "Cobb, a skilled thief who commits corporate espionage by infiltrating the subconscious of his targets is offered a chance to regain his old life as payment for a task considered to be impossible: \"inception\", the implantation of another person's idea into a target's subconscious.",
    posterPath: "/9gk7adHYeDvHkCSEqAvQNLV5Uge.jpg",
    backdropPath: "/s3TBrRGB1jav7szbG0JHaxGNQlq.jpg",
    releaseDate: "2010-07-15",
    voteAverage: 8.4,
    voteCount: 34567,
    popularity: 100.5,
    genreIds: [28, 878, 12],
    mediaType: "movie",
    originalLanguage: "en",
    runtime: 148,
    status: "Released",
    tagline: "Your mind is the scene of the crime."
  },
  {
    id: 2,
    title: "The Dark Knight",
    overview: "Batman raises the stakes in his war on crime. With the help of Lt. Jim Gordon and District Attorney Harvey Dent, Batman sets out to dismantle the remaining criminal organizations that plague the streets. The partnership proves to be effective, but they soon find themselves prey to a reign of chaos unleashed by a rising criminal mastermind known to the terrified citizens of Gotham as the Joker.",
    posterPath: "/qJ2tW6WMUDux911r6m7haRef0WH.jpg",
    backdropPath: "/hkBaDkMWbLaf8B1lsWsKX7Ew3Xq.jpg",
    releaseDate: "2008-07-14",
    voteAverage: 8.5,
    voteCount: 30678,
    popularity: 95.2,
    genreIds: [18, 28, 80, 53],
    mediaType: "movie",
    originalLanguage: "en",
    runtime: 152,
    status: "Released",
    tagline: "Why So Serious?"
  },
  {
    id: 3,
    title: "Interstellar",
    overview: "The adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.",
    posterPath: "/gEU2QniL6E8ahMcafCUyGdjIyns.jpg",
    backdropPath: "/xJHokMbljvjADYdit5fK5VQsXEG.jpg",
    releaseDate: "2014-11-05",
    voteAverage: 8.4,
    voteCount: 32561,
    popularity: 120.8,
    genreIds: [12, 18, 878],
    mediaType: "movie",
    originalLanguage: "en",
    runtime: 169,
    status: "Released",
    tagline: "Mankind was born on Earth. It was never meant to die here."
  }
];

export const MOCK_TV_SHOWS: TVShow[] = [
  {
    id: 101,
    title: "Breaking Bad",
    overview: "When Walter White, a New Mexico chemistry teacher, is diagnosed with Stage III cancer and given a prognosis of two years left to live. He becomes filled with a sense of fearlessness and an unrelenting desire to secure his family's financial future at any cost as he enters the dangerous world of drugs and crime.",
    posterPath: "/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
    backdropPath: "/tsRy63Mu5cu8etL1X7ZLyf7UP1M.jpg",
    releaseDate: "2008-01-20",
    voteAverage: 8.9,
    voteCount: 12345,
    popularity: 200.5,
    genreIds: [18, 80],
    mediaType: "tv",
    originalLanguage: "en",
    name: "Breaking Bad",
    firstAirDate: "2008-01-20",
    numberOfSeasons: 5,
    numberOfEpisodes: 62,
    status: "Ended",
    inProduction: false
  },
  {
    id: 102,
    title: "Stranger Things",
    overview: "When a young boy vanishes, a small town uncovers a mystery involving secret experiments, terrifying supernatural forces, and one strange little girl.",
    posterPath: "/49WJfeN0moxb9IPfGn8AIqMGskD.jpg",
    backdropPath: "/56v2KjBlU4XaOv9rVYkJunU560l.jpg",
    releaseDate: "2016-07-15",
    voteAverage: 8.6,
    voteCount: 15678,
    popularity: 180.2,
    genreIds: [18, 10765, 9648],
    mediaType: "tv",
    originalLanguage: "en",
    name: "Stranger Things",
    firstAirDate: "2016-07-15",
    numberOfSeasons: 4,
    numberOfEpisodes: 34,
    status: "Returning Series",
    inProduction: true
  }
];
