'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';

type TabId = 'intro' | 'features' | 'data' | 'ruvector' | 'comparison' | 'architecture' | 'research';
type Language = 'en' | 'fr';

interface LiveStats {
  totalContent: number;
  totalSeries: number;
  totalMovies: number;
  totalPatterns: number;
  avgSuccessRate: number;
  totalFeedback: number;
  avgLatency: string;
  topPatterns: Array<{ type: string; rate: number; uses: number }>;
}

// Translations
const translations = {
  en: {
    // Header
    backToApp: 'Back to App',
    aiSystemOverview: 'AI System Overview',
    dashboard: 'Dashboard',
    analytics: 'Analytics',
    settings: 'Settings',
    // Hero
    hackathonBadge: 'TV5MONDE Hackathon 2024 - Entertainment Discovery Track',
    heroTitle: 'Self-Learning Recommendation Engine',
    heroDescription: 'An AI-powered content discovery system that learns from every interaction, using advanced vector mathematics and reinforcement learning to deliver personalized recommendations.',
    contentItems: 'Content Items',
    activePatterns: 'Active Patterns',
    queryLatency: 'Query Latency',
    successRate: 'Success Rate',
    learningFrom: 'Learning from',
    userInteractions: 'user interactions',
    systemReady: 'System ready - waiting for first interactions',
    // Tabs
    introduction: 'Introduction',
    features: 'Features',
    dataVectors: 'Data & Vectors',
    ruvector: 'RuVector',
    comparison: 'Comparison',
    architecture: 'Architecture',
    research: 'Research',
    // RuVector Tab
    ruvectorTitle: 'RuVector: Your AI Memory',
    ruvectorSubtitle: 'Think of RuVector as the brain that remembers what you like',
    whatIsRuVector: 'What is RuVector?',
    whatIsRuVectorText: 'RuVector is like a super-smart librarian for TV shows and movies. Instead of organizing content by title or genre, it understands the "feeling" and "meaning" of each show.',
    howRuVectorWorks: 'How Does It Work?',
    ruvectorStep1Title: 'Every show becomes numbers',
    ruvectorStep1Desc: 'We convert each movie or series into 384 numbers that capture its essence - the mood, themes, style, and what makes it unique.',
    ruvectorStep2Title: 'Similar shows are "close" together',
    ruvectorStep2Desc: 'In this number space, shows that feel similar end up near each other. A dark crime drama will be close to other dark crime dramas.',
    ruvectorStep3Title: 'Lightning-fast matching',
    ruvectorStep3Desc: 'When you like something, RuVector instantly finds what\'s nearby in milliseconds, not seconds.',
    whyItMatters: 'Why Does This Matter?',
    whyItMattersText: 'Traditional systems use simple tags like "Action" or "Comedy". RuVector understands that a show can feel both exciting AND emotional, modern AND nostalgic. It captures the nuances that make recommendations feel personal.',
    ruvectorAnalogy: 'A Simple Analogy',
    ruvectorAnalogyText: 'Imagine you\'re at a party and want to meet people with similar interests. Traditional matching would group everyone by their job title. RuVector would understand that you love indie music, enjoy hiking, and prefer deep conversations - then introduce you to someone who shares those vibes, even if they have a completely different job.',
    ruvectorSpeed: 'Speed',
    ruvectorSpeedValue: '3ms searches',
    ruvectorSpeedDesc: 'Results in the blink of an eye',
    ruvectorAccuracy: 'Accuracy',
    ruvectorAccuracyValue: '384 dimensions',
    ruvectorAccuracyDesc: 'Captures every nuance',
    ruvectorLearning: 'Learning',
    ruvectorLearningValue: 'Real-time',
    ruvectorLearningDesc: 'Gets smarter with every click',
    underTheHood: 'Under the Hood',
    underTheHoodText: 'For the technically curious: RuVector uses PostgreSQL with pgvector extension, powered by Rust for blazing speed. It supports multiple distance metrics (cosine, euclidean, hyperbolic) and uses HNSW indexing for sub-linear search complexity.',
    rustPowered: 'Rust-Powered',
    rustPoweredDesc: 'Memory-safe, zero-cost abstractions',
    pgvectorIntegration: 'pgvector Integration',
    pgvectorDesc: 'Native PostgreSQL vector operations',
    hnswIndex: 'HNSW Indexing',
    hnswDesc: 'Approximate nearest neighbor search',
    // Introduction Tab
    theProblem: 'The Problem',
    problemText: 'Traditional recommendation systems are <strong>static</strong>. They rely on pre-computed similarities and user profiles that don\'t adapt in real-time. Users get stuck in "filter bubbles" seeing the same types of content, and the system doesn\'t learn from immediate feedback like skipping a recommendation.',
    ourSolution: 'Our Solution',
    solutionText: 'We built a <strong>self-learning recommendation engine</strong> that combines three powerful technologies:',
    vectorEmbeddings: 'Vector Embeddings',
    vectorEmbeddingsDesc: 'Content represented as 384-dimensional vectors for semantic similarity',
    qLearning: 'Q-Learning',
    qLearningDesc: 'Reinforcement learning that improves from every Watch/Skip action',
    hyperbolicVectors: 'Hyperbolic Vectors',
    hyperbolicVectorsDesc: 'Advanced geometry for better hierarchical content relationships',
    howItWorks: 'How It Works (Simple Version)',
    step1Title: 'You browse content',
    step1Desc: 'The AI shows you movies and series based on your mood or search',
    step2Title: 'You give feedback',
    step2Desc: 'Click "Watch" if interested, "Skip" if not - each click teaches the AI',
    step3Title: 'AI learns patterns',
    step3Desc: 'Q-Learning algorithm updates pattern success rates and rewards',
    step4Title: 'Recommendations improve',
    step4Desc: 'Future suggestions are boosted by patterns that worked well',
    keyMetrics: 'Key Metrics',
    live: 'Live',
    vectorDimensions: 'Vector Dimensions',
    semanticEmbedding: 'Semantic embedding',
    realTimeMeasurement: 'Real-time measurement',
    mlAlgorithms: 'ML Algorithms',
    workingInParallel: 'Working in parallel',
    learnedPatterns: 'Learned Patterns',
    userInteractionsLabel: 'User Interactions',
    patternSuccessRate: 'Pattern Success Rate',
    contentSafety: 'Content Safety & Parental Controls',
    contentSafetyText: 'We take content safety seriously. Our system includes multi-layer filtering to ensure age-appropriate recommendations.',
    audienceFiltering: 'Audience Filtering',
    contentRatings: 'Content Ratings',
    fuzzySearch: 'Intelligent Fuzzy Search',
    fuzzySearchText: 'Using PostgreSQL pg_trgm extension for typo-tolerant, fuzzy matching. No more frustration when you can\'t remember exact titles!',
    // Features Tab
    coreFeaturesCapabilities: 'Core Features & Capabilities',
    // Data Tab
    dataSourceTVDB: 'Data Source: TVDB API',
    dataSourceText: 'We use the official TVDB (TheTVDB.com) API to fetch real movie and TV series data, including metadata, artwork, and descriptions.',
    tvSeries: 'TV Series',
    movies: 'Movies',
    withEmbeddings: 'With Embeddings',
    languages: 'Languages',
    vectorEmbeddingsExplained: 'Vector Embeddings Explained',
    vectorEmbeddingsExplainedText: 'Each piece of content is converted into a <strong>384-dimensional vector</strong> - a list of 384 numbers that represent its "meaning" in mathematical space.',
    why384Dimensions: 'Why 384 Dimensions?',
    why384Text: 'More dimensions = more nuance. With 384 numbers, we can capture subtle differences like:',
    genreCombinations: 'Genre combinations (Action-Comedy vs Action-Drama)',
    toneAndMood: 'Tone and mood (dark vs lighthearted)',
    eraAndStyle: 'Era and style (80s nostalgia vs modern)',
    thematicElements: 'Thematic elements (family, revenge, love)',
    ruVectorTitle: 'RuVector: High-Performance Vector Database',
    ruVectorText: 'RuVector is a PostgreSQL extension written in Rust that provides ultra-fast vector operations. It\'s what makes our similarity searches return in milliseconds instead of seconds.',
    distanceFunctions: 'Distance Functions',
    cosineDistance: 'Cosine Distance',
    cosineDistanceDesc: 'Measures angle between vectors (default)',
    euclideanDistance: 'Euclidean Distance',
    euclideanDistanceDesc: 'Measures straight-line distance',
    hyperbolicDistance: 'Hyperbolic Distance',
    hyperbolicDistanceDesc: 'Better for hierarchical data (genres → subgenres)',
    performance: 'Performance (Live)',
    currentQueryLatency: 'Current Query Latency',
    vectorSearch: 'Vector Search',
    simdAcceleration: 'SIMD Acceleration',
    enabled: 'Enabled',
    activeLabel: 'active',
    // Comparison Tab
    traditionalVsOur: 'Traditional vs Our Approach',
    aspect: 'Aspect',
    traditionalSystems: 'Traditional Systems',
    ourSystem: 'Our System',
    learningSpeed: 'Learning Speed',
    batchUpdates: 'Batch updates (daily/weekly)',
    realTimeEveryInteraction: 'Real-time (every interaction)',
    coldStartProblem: 'Cold Start Problem',
    needsExtensiveHistory: 'Needs extensive user history',
    worksFromFirstClick: 'Works from first click via patterns',
    similarityMatching: 'Similarity Matching',
    genreTagMatching: 'Genre/tag matching (exact)',
    semanticVectors: 'Semantic vectors (meaning-based)',
    feedbackIntegration: 'Feedback Integration',
    explicitRatingsOnly: 'Explicit ratings only',
    watchSkipImplicit: 'Watch/Skip implicit signals',
    searchCapability: 'Search Capability',
    keywordMatching: 'Keyword matching',
    naturalLanguageUnderstanding: 'Natural language understanding',
    exploration: 'Exploration',
    fixedRecommendations: 'Fixed recommendations',
    epsilonGreedy: 'ε-greedy exploration (30%)',
    hierarchyUnderstanding: 'Hierarchy Understanding',
    flatCategories: 'Flat categories',
    hyperbolicGeometry: 'Hyperbolic geometry (tree-like)',
    traditionalApproach: 'Traditional Approach',
    ourApproachLabel: 'Our Approach',
    traditionalResult: 'Result: User sees same suggestions, gets bored',
    ourResult: 'Result: Personalized, improving recommendations',
    whyQLearningWins: 'Why Q-Learning Wins',
    mathBehindIt: 'The Math Behind It',
    inPlainEnglish: 'In Plain English',
    inPlainEnglishText: 'Every time you click "Watch" or "Skip", the AI updates its understanding of what works. Patterns that lead to watches get higher "Q-values" (quality scores) and are more likely to be recommended in the future. The 30% exploration rate ensures we don\'t get stuck showing only what we think you\'ll like - we occasionally try new things to learn more.',
    // Architecture Tab
    systemArchitecture: 'System Architecture',
    dataFlowWatch: 'Data Flow: Watch Action',
    keyDatabaseTables: 'Key Database Tables',
    // Research Tab
    researchFoundation: 'Research Foundation',
    researchFoundationText: 'Our system is built on established research in machine learning and information retrieval. Here are the key papers and concepts that inspired our approach:',
    optimizationsInnovations: 'Optimizations & Innovations',
    futureDirections: 'Future Directions',
    technologyStack: 'Technology Stack',
  },
  fr: {
    // Header
    backToApp: 'Retour à l\'app',
    aiSystemOverview: 'Aperçu du Système IA',
    dashboard: 'Tableau de Bord',
    analytics: 'Analytique',
    settings: 'Paramètres',
    // Hero
    hackathonBadge: 'TV5MONDE Hackathon 2024 - Piste Découverte de Divertissement',
    heroTitle: 'Moteur de Recommandation Auto-Apprenant',
    heroDescription: 'Un système de découverte de contenu alimenté par l\'IA qui apprend de chaque interaction, utilisant des mathématiques vectorielles avancées et l\'apprentissage par renforcement pour des recommandations personnalisées.',
    contentItems: 'Éléments de Contenu',
    activePatterns: 'Modèles Actifs',
    queryLatency: 'Latence de Requête',
    successRate: 'Taux de Succès',
    learningFrom: 'Apprentissage à partir de',
    userInteractions: 'interactions utilisateur',
    systemReady: 'Système prêt - en attente des premières interactions',
    // Tabs
    introduction: 'Introduction',
    features: 'Fonctionnalités',
    dataVectors: 'Données & Vecteurs',
    ruvector: 'RuVector',
    comparison: 'Comparaison',
    architecture: 'Architecture',
    research: 'Recherche',
    // RuVector Tab
    ruvectorTitle: 'RuVector : Votre Mémoire IA',
    ruvectorSubtitle: 'Pensez à RuVector comme le cerveau qui se souvient de vos préférences',
    whatIsRuVector: 'Qu\'est-ce que RuVector ?',
    whatIsRuVectorText: 'RuVector est comme un bibliothécaire super-intelligent pour les séries TV et les films. Au lieu d\'organiser le contenu par titre ou genre, il comprend le "ressenti" et le "sens" de chaque émission.',
    howRuVectorWorks: 'Comment ça fonctionne ?',
    ruvectorStep1Title: 'Chaque émission devient des nombres',
    ruvectorStep1Desc: 'Nous convertissons chaque film ou série en 384 nombres qui capturent son essence - l\'ambiance, les thèmes, le style et ce qui le rend unique.',
    ruvectorStep2Title: 'Les émissions similaires sont "proches"',
    ruvectorStep2Desc: 'Dans cet espace numérique, les émissions qui se ressemblent finissent proches les unes des autres. Un drame policier sombre sera proche d\'autres drames policiers sombres.',
    ruvectorStep3Title: 'Correspondance ultra-rapide',
    ruvectorStep3Desc: 'Quand vous aimez quelque chose, RuVector trouve instantanément ce qui est proche en millisecondes, pas en secondes.',
    whyItMatters: 'Pourquoi c\'est important ?',
    whyItMattersText: 'Les systèmes traditionnels utilisent des étiquettes simples comme "Action" ou "Comédie". RuVector comprend qu\'une émission peut être à la fois excitante ET émouvante, moderne ET nostalgique. Il capture les nuances qui rendent les recommandations personnelles.',
    ruvectorAnalogy: 'Une Analogie Simple',
    ruvectorAnalogyText: 'Imaginez que vous êtes à une fête et que vous voulez rencontrer des gens avec des intérêts similaires. La correspondance traditionnelle regrouperait tout le monde par intitulé de poste. RuVector comprendrait que vous aimez la musique indie, aimez la randonnée et préférez les conversations profondes - puis vous présenterait quelqu\'un qui partage ces vibrations, même s\'il a un travail complètement différent.',
    ruvectorSpeed: 'Vitesse',
    ruvectorSpeedValue: 'Recherches en 3ms',
    ruvectorSpeedDesc: 'Résultats en un clin d\'œil',
    ruvectorAccuracy: 'Précision',
    ruvectorAccuracyValue: '384 dimensions',
    ruvectorAccuracyDesc: 'Capture chaque nuance',
    ruvectorLearning: 'Apprentissage',
    ruvectorLearningValue: 'Temps réel',
    ruvectorLearningDesc: 'Devient plus intelligent à chaque clic',
    underTheHood: 'Sous le Capot',
    underTheHoodText: 'Pour les curieux techniques : RuVector utilise PostgreSQL avec l\'extension pgvector, propulsé par Rust pour une vitesse fulgurante. Il supporte plusieurs métriques de distance (cosinus, euclidienne, hyperbolique) et utilise l\'indexation HNSW pour une complexité de recherche sous-linéaire.',
    rustPowered: 'Propulsé par Rust',
    rustPoweredDesc: 'Sécurité mémoire, abstractions sans coût',
    pgvectorIntegration: 'Intégration pgvector',
    pgvectorDesc: 'Opérations vectorielles PostgreSQL natives',
    hnswIndex: 'Indexation HNSW',
    hnswDesc: 'Recherche approximative du plus proche voisin',
    // Introduction Tab
    theProblem: 'Le Problème',
    problemText: 'Les systèmes de recommandation traditionnels sont <strong>statiques</strong>. Ils reposent sur des similarités pré-calculées et des profils utilisateur qui ne s\'adaptent pas en temps réel. Les utilisateurs restent coincés dans des "bulles de filtre" voyant les mêmes types de contenu, et le système n\'apprend pas des retours immédiats comme le fait de passer une recommandation.',
    ourSolution: 'Notre Solution',
    solutionText: 'Nous avons construit un <strong>moteur de recommandation auto-apprenant</strong> qui combine trois technologies puissantes :',
    vectorEmbeddings: 'Embeddings Vectoriels',
    vectorEmbeddingsDesc: 'Contenu représenté sous forme de vecteurs à 384 dimensions pour la similarité sémantique',
    qLearning: 'Q-Learning',
    qLearningDesc: 'Apprentissage par renforcement qui s\'améliore à chaque action Regarder/Passer',
    hyperbolicVectors: 'Vecteurs Hyperboliques',
    hyperbolicVectorsDesc: 'Géométrie avancée pour de meilleures relations hiérarchiques de contenu',
    howItWorks: 'Comment Ça Marche (Version Simple)',
    step1Title: 'Vous parcourez le contenu',
    step1Desc: 'L\'IA vous montre des films et séries basés sur votre humeur ou recherche',
    step2Title: 'Vous donnez votre avis',
    step2Desc: 'Cliquez "Regarder" si intéressé, "Passer" sinon - chaque clic enseigne l\'IA',
    step3Title: 'L\'IA apprend les modèles',
    step3Desc: 'L\'algorithme Q-Learning met à jour les taux de succès et récompenses des modèles',
    step4Title: 'Les recommandations s\'améliorent',
    step4Desc: 'Les suggestions futures sont boostées par les modèles qui ont bien fonctionné',
    keyMetrics: 'Métriques Clés',
    live: 'En Direct',
    vectorDimensions: 'Dimensions Vectorielles',
    semanticEmbedding: 'Embedding sémantique',
    realTimeMeasurement: 'Mesure en temps réel',
    mlAlgorithms: 'Algorithmes ML',
    workingInParallel: 'Travaillant en parallèle',
    learnedPatterns: 'Modèles Appris',
    userInteractionsLabel: 'Interactions Utilisateur',
    patternSuccessRate: 'Taux de Succès des Modèles',
    contentSafety: 'Sécurité du Contenu & Contrôle Parental',
    contentSafetyText: 'Nous prenons la sécurité du contenu au sérieux. Notre système inclut un filtrage multicouche pour assurer des recommandations adaptées à l\'âge.',
    audienceFiltering: 'Filtrage par Audience',
    contentRatings: 'Classifications de Contenu',
    fuzzySearch: 'Recherche Floue Intelligente',
    fuzzySearchText: 'Utilisant l\'extension PostgreSQL pg_trgm pour une correspondance floue tolérante aux fautes de frappe. Plus de frustration quand vous ne vous souvenez pas des titres exacts !',
    // Features Tab
    coreFeaturesCapabilities: 'Fonctionnalités et Capacités Principales',
    // Data Tab
    dataSourceTVDB: 'Source de Données : API TVDB',
    dataSourceText: 'Nous utilisons l\'API officielle TVDB (TheTVDB.com) pour récupérer de vraies données de films et séries TV, incluant métadonnées, illustrations et descriptions.',
    tvSeries: 'Séries TV',
    movies: 'Films',
    withEmbeddings: 'Avec Embeddings',
    languages: 'Langues',
    vectorEmbeddingsExplained: 'Les Embeddings Vectoriels Expliqués',
    vectorEmbeddingsExplainedText: 'Chaque contenu est converti en un <strong>vecteur à 384 dimensions</strong> - une liste de 384 nombres qui représentent sa "signification" dans l\'espace mathématique.',
    why384Dimensions: 'Pourquoi 384 Dimensions ?',
    why384Text: 'Plus de dimensions = plus de nuance. Avec 384 nombres, nous pouvons capturer des différences subtiles comme :',
    genreCombinations: 'Combinaisons de genres (Action-Comédie vs Action-Drame)',
    toneAndMood: 'Ton et ambiance (sombre vs léger)',
    eraAndStyle: 'Époque et style (nostalgie 80s vs moderne)',
    thematicElements: 'Éléments thématiques (famille, vengeance, amour)',
    ruVectorTitle: 'RuVector : Base de Données Vectorielle Haute Performance',
    ruVectorText: 'RuVector est une extension PostgreSQL écrite en Rust qui fournit des opérations vectorielles ultra-rapides. C\'est ce qui permet à nos recherches de similarité de retourner en millisecondes au lieu de secondes.',
    distanceFunctions: 'Fonctions de Distance',
    cosineDistance: 'Distance Cosinus',
    cosineDistanceDesc: 'Mesure l\'angle entre vecteurs (par défaut)',
    euclideanDistance: 'Distance Euclidienne',
    euclideanDistanceDesc: 'Mesure la distance en ligne droite',
    hyperbolicDistance: 'Distance Hyperbolique',
    hyperbolicDistanceDesc: 'Meilleure pour les données hiérarchiques (genres → sous-genres)',
    performance: 'Performance (En Direct)',
    currentQueryLatency: 'Latence de Requête Actuelle',
    vectorSearch: 'Recherche Vectorielle',
    simdAcceleration: 'Accélération SIMD',
    enabled: 'Activée',
    activeLabel: 'actifs',
    // Comparison Tab
    traditionalVsOur: 'Approche Traditionnelle vs La Nôtre',
    aspect: 'Aspect',
    traditionalSystems: 'Systèmes Traditionnels',
    ourSystem: 'Notre Système',
    learningSpeed: 'Vitesse d\'Apprentissage',
    batchUpdates: 'Mises à jour par lots (quotidien/hebdomadaire)',
    realTimeEveryInteraction: 'Temps réel (chaque interaction)',
    coldStartProblem: 'Problème du Démarrage à Froid',
    needsExtensiveHistory: 'Nécessite un historique utilisateur étendu',
    worksFromFirstClick: 'Fonctionne dès le premier clic via les modèles',
    similarityMatching: 'Correspondance de Similarité',
    genreTagMatching: 'Correspondance genre/tag (exacte)',
    semanticVectors: 'Vecteurs sémantiques (basé sur le sens)',
    feedbackIntegration: 'Intégration des Retours',
    explicitRatingsOnly: 'Notes explicites uniquement',
    watchSkipImplicit: 'Signaux implicites Regarder/Passer',
    searchCapability: 'Capacité de Recherche',
    keywordMatching: 'Correspondance par mots-clés',
    naturalLanguageUnderstanding: 'Compréhension du langage naturel',
    exploration: 'Exploration',
    fixedRecommendations: 'Recommandations fixes',
    epsilonGreedy: 'Exploration ε-greedy (30%)',
    hierarchyUnderstanding: 'Compréhension Hiérarchique',
    flatCategories: 'Catégories plates',
    hyperbolicGeometry: 'Géométrie hyperbolique (arborescente)',
    traditionalApproach: 'Approche Traditionnelle',
    ourApproachLabel: 'Notre Approche',
    traditionalResult: 'Résultat : L\'utilisateur voit les mêmes suggestions, s\'ennuie',
    ourResult: 'Résultat : Recommandations personnalisées et améliorées',
    whyQLearningWins: 'Pourquoi le Q-Learning Gagne',
    mathBehindIt: 'Les Mathématiques Derrière',
    inPlainEnglish: 'En Termes Simples',
    inPlainEnglishText: 'Chaque fois que vous cliquez "Regarder" ou "Passer", l\'IA met à jour sa compréhension de ce qui fonctionne. Les modèles qui mènent à des visionnages obtiennent des "Q-values" (scores de qualité) plus élevées et sont plus susceptibles d\'être recommandés à l\'avenir. Le taux d\'exploration de 30% assure que nous ne restons pas bloqués à montrer uniquement ce que nous pensons que vous aimerez - nous essayons occasionnellement de nouvelles choses pour en apprendre plus.',
    // Architecture Tab
    systemArchitecture: 'Architecture du Système',
    dataFlowWatch: 'Flux de Données : Action Regarder',
    keyDatabaseTables: 'Tables de Base de Données Clés',
    // Research Tab
    researchFoundation: 'Fondation de Recherche',
    researchFoundationText: 'Notre système est construit sur des recherches établies en apprentissage automatique et recherche d\'information. Voici les articles et concepts clés qui ont inspiré notre approche :',
    optimizationsInnovations: 'Optimisations & Innovations',
    futureDirections: 'Directions Futures',
    technologyStack: 'Stack Technologique',
  },
};

type TranslationKey = keyof typeof translations.en;

export default function AboutPage() {
  const [activeTab, setActiveTab] = useState<TabId>('intro');
  const [language, setLanguage] = useState<Language>('en');
  const [showSettings, setShowSettings] = useState(false);
  const [liveStats, setLiveStats] = useState<LiveStats | null>(null);
  const [animatedStats, setAnimatedStats] = useState({
    content: 0,
    patterns: 0,
    feedback: 0,
    successRate: 0,
  });

  const t = translations[language];

  // Fetch live stats
  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch('/api/stats');
        const data = await response.json();
        setLiveStats({
          totalContent: data.stats?.totalContent || 0,
          totalSeries: data.stats?.totalSeries || 0,
          totalMovies: data.stats?.totalMovies || 0,
          totalPatterns: data.stats?.totalPatterns || 0,
          avgSuccessRate: data.stats?.avgSuccessRate || 0,
          totalFeedback: data.stats?.totalFeedback || 0,
          avgLatency: data.learning?.vectorSpace?.avgSearchLatency || '3.2ms',
          topPatterns: data.stats?.topPatterns || [],
        });
      } catch (error) {
        console.error('Failed to fetch stats:', error);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 15000);
    return () => clearInterval(interval);
  }, []);

  // Animate counters when stats arrive
  useEffect(() => {
    if (liveStats) {
      const duration = 2000;
      const steps = 60;
      const interval = duration / steps;
      let step = 0;

      const timer = setInterval(() => {
        step++;
        const progress = step / steps;
        const easeOut = 1 - Math.pow(1 - progress, 3);

        setAnimatedStats({
          content: Math.round(liveStats.totalContent * easeOut),
          patterns: Math.round(liveStats.totalPatterns * easeOut),
          feedback: Math.round(liveStats.totalFeedback * easeOut),
          successRate: Math.round(liveStats.avgSuccessRate * 100 * easeOut),
        });

        if (step >= steps) clearInterval(timer);
      }, interval);

      return () => clearInterval(timer);
    }
  }, [liveStats]);

  const tabs: { id: TabId; labelKey: keyof typeof translations.en; icon: JSX.Element }[] = [
    {
      id: 'intro',
      labelKey: 'introduction',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      ),
    },
    {
      id: 'features',
      labelKey: 'features',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
        </svg>
      ),
    },
    {
      id: 'data',
      labelKey: 'dataVectors',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
        </svg>
      ),
    },
    {
      id: 'ruvector',
      labelKey: 'ruvector',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
      ),
    },
    {
      id: 'comparison',
      labelKey: 'comparison',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
      ),
    },
    {
      id: 'architecture',
      labelKey: 'architecture',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      ),
    },
    {
      id: 'research',
      labelKey: 'research',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
        </svg>
      ),
    },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-zinc-950 via-zinc-900 to-zinc-950">
      {/* Sticky Navigation */}
      <nav className="sticky top-0 z-50 glass-card border-b border-zinc-800/50 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-3 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg feature-icon flex items-center justify-center">
              <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
            <span className="font-bold text-white hidden sm:inline">TVDB Smart</span>
          </Link>

          <div className="flex items-center gap-2 sm:gap-4">
            <Link
              href="/"
              className="px-3 py-1.5 rounded-lg text-xs sm:text-sm font-medium text-zinc-400 hover:text-white hover:bg-zinc-800 transition-all"
            >
              Dashboard
            </Link>
            <Link
              href="/analytics"
              className="px-3 py-1.5 rounded-lg text-xs sm:text-sm font-medium text-zinc-400 hover:text-white hover:bg-zinc-800 transition-all"
            >
              Analytics
            </Link>
            <Link
              href="/about"
              className="px-3 py-1.5 rounded-lg text-xs sm:text-sm font-medium text-emerald-400 bg-emerald-500/10 border border-emerald-500/20"
            >
              About
            </Link>
            <button
              onClick={() => setLanguage(language === 'en' ? 'fr' : 'en')}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 border border-zinc-700 hover:border-emerald-500/50 transition-colors"
            >
              <span className="text-lg">{language === 'en' ? '🇬🇧' : '🇫🇷'}</span>
              <span className="text-sm font-medium text-white">{language.toUpperCase()}</span>
            </button>
          </div>
        </div>
      </nav>

      {/* Settings Modal */}
      {showSettings && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
          <div className="glass-card rounded-2xl p-6 max-w-md w-full mx-4 border border-zinc-700">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-white">{t.settings}</h2>
              <button
                onClick={() => setShowSettings(false)}
                className="p-2 text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="space-y-4">
              {/* Language Setting */}
              <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-white font-medium">{language === 'en' ? 'Language' : 'Langue'}</div>
                    <div className="text-zinc-500 text-sm">{language === 'en' ? 'Choose your preferred language' : 'Choisissez votre langue'}</div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => setLanguage('en')}
                      className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                        language === 'en'
                          ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                          : 'bg-zinc-700 text-zinc-400 hover:text-white'
                      }`}
                    >
                      English
                    </button>
                    <button
                      onClick={() => setLanguage('fr')}
                      className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                        language === 'fr'
                          ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                          : 'bg-zinc-700 text-zinc-400 hover:text-white'
                      }`}
                    >
                      Francais
                    </button>
                  </div>
                </div>
              </div>

              {/* Navigation Links in Modal (for mobile) */}
              <div className="md:hidden space-y-2">
                <Link
                  href="/"
                  className="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-zinc-400 hover:text-white hover:border-emerald-500/30 transition-colors"
                  onClick={() => setShowSettings(false)}
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                  </svg>
                  {t.dashboard}
                </Link>
                <Link
                  href="/analytics"
                  className="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-zinc-400 hover:text-white hover:border-emerald-500/30 transition-colors"
                  onClick={() => setShowSettings(false)}
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                  {t.analytics}
                </Link>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Hero with Live Stats */}
      <div className="relative overflow-hidden">
        <div className="absolute inset-0 opacity-30">
          <div className="absolute top-20 left-20 w-72 h-72 rounded-full bg-emerald-500/20 blur-3xl animate-pulse" style={{ animationDuration: '4s' }} />
          <div className="absolute bottom-20 right-20 w-96 h-96 rounded-full bg-blue-500/10 blur-3xl animate-pulse" style={{ animationDuration: '6s', animationDelay: '2s' }} />
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 rounded-full bg-purple-500/10 blur-3xl animate-pulse" style={{ animationDuration: '5s', animationDelay: '1s' }} />
        </div>
        <div className="relative max-w-6xl mx-auto px-6 py-16 text-center">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm font-medium mb-6">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            {t.hackathonBadge}
          </div>
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
            {t.heroTitle}
          </h1>
          <p className="text-xl text-zinc-400 max-w-3xl mx-auto mb-8">
            {t.heroDescription}
          </p>

          {/* Animated Live Stats Bar */}
          <div className="flex flex-wrap items-center justify-center gap-4 md:gap-8">
            <div className="group relative">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-emerald-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-emerald-400 transition-transform group-hover:scale-110">
                  {animatedStats.content.toLocaleString()}
                </div>
                <div className="text-zinc-500 text-sm">{t.contentItems}</div>
                <div className="absolute -top-2 -right-2 w-4 h-4 bg-emerald-500 rounded-full animate-ping opacity-75" />
              </div>
            </div>
            <div className="group">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-blue-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-blue-400 transition-transform group-hover:scale-110">
                  {animatedStats.patterns}
                </div>
                <div className="text-zinc-500 text-sm">{t.activePatterns}</div>
              </div>
            </div>
            <div className="group">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-purple-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-purple-400 transition-transform group-hover:scale-110">
                  {liveStats?.avgLatency || '~3ms'}
                </div>
                <div className="text-zinc-500 text-sm">{t.queryLatency}</div>
              </div>
            </div>
            <div className="group relative">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-yellow-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-yellow-400 transition-transform group-hover:scale-110">
                  {animatedStats.successRate}%
                </div>
                <div className="text-zinc-500 text-sm">{t.successRate}</div>
                {(liveStats?.avgSuccessRate || 0) > 0.5 && (
                  <div className="absolute -top-2 -right-2 px-2 py-0.5 bg-emerald-500 rounded-full text-[10px] text-white font-bold animate-bounce">
                    LIVE
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Learning indicator */}
          {liveStats && (
            <div className="mt-6 inline-flex items-center gap-2 text-sm text-zinc-400">
              <span className="relative flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
              </span>
              {liveStats.totalFeedback > 0 ? (
                <span>{t.learningFrom} <span className="text-emerald-400 font-semibold">{liveStats.totalFeedback}</span> {t.userInteractions}</span>
              ) : (
                <span>{t.systemReady}</span>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-zinc-800 sticky top-0 bg-zinc-950/95 backdrop-blur-sm z-10">
        <div className="max-w-6xl mx-auto px-6">
          <div className="flex gap-1 overflow-x-auto py-2">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg text-sm font-medium whitespace-nowrap transition-all ${
                  activeTab === tab.id
                    ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                    : 'text-zinc-400 hover:text-white hover:bg-zinc-800/50'
                }`}
              >
                {tab.icon}
                {t[tab.labelKey]}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-6xl mx-auto px-6 py-12">
        {activeTab === 'intro' && <IntroductionTab liveStats={liveStats} animatedStats={animatedStats} t={t} />}
        {activeTab === 'features' && <FeaturesTab t={t} />}
        {activeTab === 'data' && <DataTab liveStats={liveStats} t={t} />}
        {activeTab === 'ruvector' && <RuVectorTab liveStats={liveStats} t={t} />}
        {activeTab === 'comparison' && <ComparisonTab t={t} />}
        {activeTab === 'architecture' && <ArchitectureTab t={t} />}
        {activeTab === 'research' && <ResearchTab t={t} />}
      </div>
    </div>
  );
}

interface IntroductionTabProps {
  liveStats: LiveStats | null;
  animatedStats: { content: number; patterns: number; feedback: number; successRate: number };
  t: typeof translations.en;
}

function IntroductionTab({ liveStats, animatedStats, t }: IntroductionTabProps) {
  return (
    <div className="space-y-12">
      {/* Problem Statement */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </span>
          {t.theProblem}
        </h2>
        <p className="text-zinc-400 text-lg leading-relaxed" dangerouslySetInnerHTML={{ __html: t.problemText }} />
      </section>

      {/* Solution */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </span>
          {t.ourSolution}
        </h2>
        <p className="text-zinc-400 text-lg leading-relaxed mb-6" dangerouslySetInnerHTML={{ __html: t.solutionText }} />
        <div className="grid md:grid-cols-3 gap-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-3xl mb-2">🧮</div>
            <h3 className="text-white font-semibold mb-1">{t.vectorEmbeddings}</h3>
            <p className="text-zinc-500 text-sm">{t.vectorEmbeddingsDesc}</p>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-3xl mb-2">🎯</div>
            <h3 className="text-white font-semibold mb-1">{t.qLearning}</h3>
            <p className="text-zinc-500 text-sm">{t.qLearningDesc}</p>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-3xl mb-2">🌐</div>
            <h3 className="text-white font-semibold mb-1">{t.hyperbolicVectors}</h3>
            <p className="text-zinc-500 text-sm">{t.hyperbolicVectorsDesc}</p>
          </div>
        </div>
      </section>

      {/* How It Works - Simple */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </span>
          {t.howItWorks}
        </h2>
        <div className="space-y-6">
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">1</div>
            <div>
              <h3 className="text-white font-semibold text-lg">{t.step1Title}</h3>
              <p className="text-zinc-400">{t.step1Desc}</p>
            </div>
          </div>
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">2</div>
            <div>
              <h3 className="text-white font-semibold text-lg">{t.step2Title}</h3>
              <p className="text-zinc-400">{t.step2Desc}</p>
            </div>
          </div>
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">3</div>
            <div>
              <h3 className="text-white font-semibold text-lg">{t.step3Title}</h3>
              <p className="text-zinc-400">{t.step3Desc}</p>
            </div>
          </div>
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">4</div>
            <div>
              <h3 className="text-white font-semibold text-lg">{t.step4Title}</h3>
              <p className="text-zinc-400">{t.step4Desc}</p>
            </div>
          </div>
        </div>
      </section>

      {/* Key Metrics - Animated with Live Data */}
      <section>
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-3">
          {t.keyMetrics}
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
          <span className="text-xs font-normal text-zinc-500">{t.live}</span>
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="glass-card rounded-xl p-6 text-center group hover:border-emerald-500/30 transition-all">
            <div className="text-3xl font-bold text-emerald-400 mb-1 transition-transform group-hover:scale-110">
              {animatedStats.content.toLocaleString()}
            </div>
            <div className="text-zinc-500 text-sm">{t.contentItems}</div>
            <div className="text-xs text-zinc-600 mt-1">
              {liveStats ? `${liveStats.totalSeries} ${t.tvSeries.toLowerCase()} + ${liveStats.totalMovies} ${t.movies.toLowerCase()}` : 'Loading...'}
            </div>
          </div>
          <div className="glass-card rounded-xl p-6 text-center group hover:border-blue-500/30 transition-all">
            <div className="text-3xl font-bold text-blue-400 mb-1">384</div>
            <div className="text-zinc-500 text-sm">{t.vectorDimensions}</div>
            <div className="text-xs text-zinc-600 mt-1">{t.semanticEmbedding}</div>
          </div>
          <div className="glass-card rounded-xl p-6 text-center group hover:border-purple-500/30 transition-all">
            <div className="text-3xl font-bold text-purple-400 mb-1 transition-transform group-hover:scale-110">
              {liveStats?.avgLatency || '~3ms'}
            </div>
            <div className="text-zinc-500 text-sm">{t.queryLatency}</div>
            <div className="text-xs text-emerald-500 mt-1">{t.realTimeMeasurement}</div>
          </div>
          <div className="glass-card rounded-xl p-6 text-center group hover:border-yellow-500/30 transition-all">
            <div className="text-3xl font-bold text-yellow-400 mb-1">6</div>
            <div className="text-zinc-500 text-sm">{t.mlAlgorithms}</div>
            <div className="text-xs text-zinc-600 mt-1">{t.workingInParallel}</div>
          </div>
        </div>

        {/* Additional animated stats */}
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mt-4">
          <div className="glass-card rounded-xl p-4 text-center">
            <div className="text-2xl font-bold text-cyan-400 mb-1">{animatedStats.patterns}</div>
            <div className="text-zinc-500 text-sm">{t.learnedPatterns}</div>
          </div>
          <div className="glass-card rounded-xl p-4 text-center">
            <div className="text-2xl font-bold text-pink-400 mb-1">{animatedStats.feedback}</div>
            <div className="text-zinc-500 text-sm">{t.userInteractionsLabel}</div>
          </div>
          <div className="glass-card rounded-xl p-4 text-center col-span-2 md:col-span-1">
            <div className="text-2xl font-bold text-orange-400 mb-1">{animatedStats.successRate}%</div>
            <div className="text-zinc-500 text-sm">{t.patternSuccessRate}</div>
          </div>
        </div>
      </section>

      {/* Content Safety */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </span>
          {t.contentSafety}
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          {t.contentSafetyText}
        </p>
        <div className="grid md:grid-cols-2 gap-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-emerald-400 font-semibold mb-2">{t.audienceFiltering}</div>
            <ul className="text-zinc-400 text-sm space-y-1">
              <li>• Kids: G-rated only, family genres, no violence</li>
              <li>• Family: G/PG rated, no horror or mature themes</li>
              <li>• Teens: Up to PG-13, excludes R-rated</li>
              <li>• Adults: Full catalog access</li>
            </ul>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-emerald-400 font-semibold mb-2">{t.contentRatings}</div>
            <ul className="text-zinc-400 text-sm space-y-1">
              <li>• G: 331 titles (general audiences)</li>
              <li>• PG: 14 titles (parental guidance)</li>
              <li>• PG-13: 1,364 titles (teens and up)</li>
              <li>• R: 506 titles (adults only)</li>
            </ul>
          </div>
        </div>
      </section>

      {/* Fuzzy Search */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </span>
          {t.fuzzySearch}
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          {t.fuzzySearchText}
        </p>
        <div className="bg-zinc-900 rounded-xl p-6 font-mono text-sm">
          <div className="text-zinc-500 mb-2">// Example fuzzy matches:</div>
          <div className="flex justify-between text-sm mb-1">
            <span className="text-blue-400">"starwars"</span>
            <span className="text-zinc-500">→</span>
            <span className="text-emerald-400">"Star Wars" (58% similarity)</span>
          </div>
          <div className="flex justify-between text-sm mb-1">
            <span className="text-blue-400">"gameofthrones"</span>
            <span className="text-zinc-500">→</span>
            <span className="text-emerald-400">"Game of Thrones" (65% similarity)</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-blue-400">"breakingbad"</span>
            <span className="text-zinc-500">→</span>
            <span className="text-emerald-400">"Breaking Bad" (72% similarity)</span>
          </div>
        </div>
      </section>
    </div>
  );
}

interface FeaturesTabProps {
  t: typeof translations.en;
}

function FeaturesTab({ t }: FeaturesTabProps) {
  const features = [
    {
      icon: '🧠',
      title: 'Advanced Multi-Algorithm Learning',
      description: 'Six sophisticated ML algorithms working together: Q-Learning, Double Q-Learning, Thompson Sampling, UCB1, LinUCB, and Prioritized Experience Replay.',
      details: [
        'Thompson Sampling: Bayesian bandits for exploration',
        'UCB1: Upper Confidence Bound optimization',
        'Double Q-Learning: Prevents overestimation bias',
      ],
    },
    {
      icon: '📊',
      title: 'Pattern Recognition',
      description: 'Automatically identifies successful recommendation patterns like "Comedy + Romance for date night" or "Action + Thriller for excitement seekers".',
      details: [
        'Top pattern: comedy_romance (100% success)',
        'Genre combos: drama_crime, sci-fi_action',
        'Boosts future recommendations using top patterns',
      ],
    },
    {
      icon: '🔍',
      title: 'Fuzzy Search with pg_trgm',
      description: 'Typo-tolerant search using PostgreSQL trigram similarity. "starwars" finds "Star Wars", "breakingbad" finds "Breaking Bad".',
      details: [
        'Trigram similarity matching',
        'Results ordered by relevance score',
        'Handles missing spaces and typos',
      ],
    },
    {
      icon: '🛡️',
      title: 'Content Safety Filtering',
      description: 'Multi-layer parental controls ensure age-appropriate recommendations. Kids never see R-rated content.',
      details: [
        'Audience levels: Kids, Family, Teens, Adults',
        'Rating filtering: G, PG, PG-13, R',
        'Content overview screening for mature themes',
      ],
    },
    {
      icon: '📈',
      title: 'Analytics Dashboard',
      description: 'Interactive visualization with force-directed graph, real-time metrics, pattern treemap, and activity stream.',
      details: [
        'Genre network: 19 nodes, 50 connections',
        'Live auto-refresh every 5 seconds',
        '4 tabs: Overview, Learning, Network, Activity',
      ],
    },
    {
      icon: '🎭',
      title: 'Mood-Based Discovery',
      description: 'Quick mood selectors that filter content by emotional intent rather than just genre.',
      details: [
        'Funny, Exciting, Romantic, Scary, Thoughtful, Relaxing',
        'Maps moods to genre combinations',
        'Learns which moods lead to watches',
      ],
    },
    {
      icon: '⚡',
      title: 'Vector Optimization',
      description: 'HNSW indexing for 150x faster search and scalar quantization for 4-32x memory reduction.',
      details: [
        'HNSW: M=16, efConstruction=200',
        'Scalar quantization: 8-bit precision',
        'SIMD-accelerated distance calculations',
      ],
    },
    {
      icon: '🔄',
      title: 'Prioritized Experience Replay',
      description: 'Learns more from surprising or important experiences using TD-error prioritization.',
      details: [
        'Priority = |TD-error|^α for sampling',
        'Importance sampling weights β',
        'Buffer size: 10,000 experiences',
      ],
    },
  ];

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-white mb-6">{t.coreFeaturesCapabilities}</h2>
      <div className="grid md:grid-cols-2 gap-6">
        {features.map((feature, index) => (
          <div key={index} className="glass-card rounded-2xl p-6 hover:border-emerald-500/30 transition-colors">
            <div className="text-4xl mb-4">{feature.icon}</div>
            <h3 className="text-xl font-semibold text-white mb-2">{feature.title}</h3>
            <p className="text-zinc-400 mb-4">{feature.description}</p>
            <ul className="space-y-2">
              {feature.details.map((detail, i) => (
                <li key={i} className="flex items-center gap-2 text-sm text-zinc-500">
                  <svg className="w-4 h-4 text-emerald-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                  {detail}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </div>
  );
}

interface DataTabProps {
  liveStats: LiveStats | null;
  t: typeof translations.en;
}

interface RuVectorTabProps {
  liveStats: LiveStats | null;
  t: typeof translations.en;
}

function RuVectorTab({ liveStats, t }: RuVectorTabProps) {
  return (
    <div className="space-y-12">
      {/* Hero Section */}
      <section className="glass-card rounded-2xl p-8 border border-emerald-500/20">
        <div className="flex items-center gap-4 mb-6">
          <span className="w-16 h-16 rounded-2xl bg-gradient-to-br from-emerald-500/20 to-cyan-500/20 flex items-center justify-center">
            <svg className="w-8 h-8 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
          </span>
          <div>
            <h2 className="text-3xl font-bold text-white">{t.ruvectorTitle}</h2>
            <p className="text-emerald-400 text-lg">{t.ruvectorSubtitle}</p>
          </div>
        </div>
      </section>

      {/* What is RuVector */}
      <section className="glass-card rounded-2xl p-8">
        <h3 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </span>
          {t.whatIsRuVector}
        </h3>
        <p className="text-zinc-300 text-lg leading-relaxed">
          {t.whatIsRuVectorText}
        </p>
      </section>

      {/* How It Works - Steps */}
      <section className="glass-card rounded-2xl p-8">
        <h3 className="text-2xl font-bold text-white mb-6 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </span>
          {t.howRuVectorWorks}
        </h3>

        <div className="space-y-6">
          {/* Step 1 */}
          <div className="flex gap-4 items-start">
            <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-emerald-500/20 flex items-center justify-center text-emerald-400 font-bold text-xl">1</div>
            <div>
              <h4 className="text-lg font-semibold text-white mb-1">{t.ruvectorStep1Title}</h4>
              <p className="text-zinc-400">{t.ruvectorStep1Desc}</p>
            </div>
          </div>

          {/* Arrow */}
          <div className="flex justify-center">
            <svg className="w-6 h-6 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
            </svg>
          </div>

          {/* Step 2 */}
          <div className="flex gap-4 items-start">
            <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-cyan-500/20 flex items-center justify-center text-cyan-400 font-bold text-xl">2</div>
            <div>
              <h4 className="text-lg font-semibold text-white mb-1">{t.ruvectorStep2Title}</h4>
              <p className="text-zinc-400">{t.ruvectorStep2Desc}</p>
            </div>
          </div>

          {/* Arrow */}
          <div className="flex justify-center">
            <svg className="w-6 h-6 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
            </svg>
          </div>

          {/* Step 3 */}
          <div className="flex gap-4 items-start">
            <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-purple-500/20 flex items-center justify-center text-purple-400 font-bold text-xl">3</div>
            <div>
              <h4 className="text-lg font-semibold text-white mb-1">{t.ruvectorStep3Title}</h4>
              <p className="text-zinc-400">{t.ruvectorStep3Desc}</p>
            </div>
          </div>
        </div>
      </section>

      {/* Why It Matters */}
      <section className="glass-card rounded-2xl p-8">
        <h3 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
            </svg>
          </span>
          {t.whyItMatters}
        </h3>
        <p className="text-zinc-300 text-lg leading-relaxed">
          {t.whyItMattersText}
        </p>
      </section>

      {/* Simple Analogy */}
      <section className="glass-card rounded-2xl p-8 bg-gradient-to-br from-indigo-500/10 to-purple-500/10 border border-indigo-500/20">
        <h3 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
          </span>
          {t.ruvectorAnalogy}
        </h3>
        <p className="text-zinc-300 text-lg leading-relaxed italic">
          "{t.ruvectorAnalogyText}"
        </p>
      </section>

      {/* Stats Cards */}
      <section className="grid md:grid-cols-3 gap-6">
        {/* Speed */}
        <div className="glass-card rounded-2xl p-6 text-center group hover:border-emerald-500/30 transition-all">
          <div className="w-14 h-14 mx-auto rounded-xl bg-emerald-500/20 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
            <svg className="w-7 h-7 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <h4 className="text-lg font-semibold text-white mb-1">{t.ruvectorSpeed}</h4>
          <p className="text-2xl font-bold text-emerald-400 mb-2">{liveStats?.avgLatency || t.ruvectorSpeedValue}</p>
          <p className="text-sm text-zinc-500">{t.ruvectorSpeedDesc}</p>
        </div>

        {/* Accuracy */}
        <div className="glass-card rounded-2xl p-6 text-center group hover:border-cyan-500/30 transition-all">
          <div className="w-14 h-14 mx-auto rounded-xl bg-cyan-500/20 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
            <svg className="w-7 h-7 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h4 className="text-lg font-semibold text-white mb-1">{t.ruvectorAccuracy}</h4>
          <p className="text-2xl font-bold text-cyan-400 mb-2">{t.ruvectorAccuracyValue}</p>
          <p className="text-sm text-zinc-500">{t.ruvectorAccuracyDesc}</p>
        </div>

        {/* Learning */}
        <div className="glass-card rounded-2xl p-6 text-center group hover:border-purple-500/30 transition-all">
          <div className="w-14 h-14 mx-auto rounded-xl bg-purple-500/20 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
            <svg className="w-7 h-7 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
          </div>
          <h4 className="text-lg font-semibold text-white mb-1">{t.ruvectorLearning}</h4>
          <p className="text-2xl font-bold text-purple-400 mb-2">{t.ruvectorLearningValue}</p>
          <p className="text-sm text-zinc-500">{t.ruvectorLearningDesc}</p>
        </div>
      </section>

      {/* Under the Hood */}
      <section className="glass-card rounded-2xl p-8">
        <h3 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-zinc-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </span>
          {t.underTheHood}
        </h3>
        <p className="text-zinc-400 text-lg mb-6">
          {t.underTheHoodText}
        </p>

        <div className="grid md:grid-cols-3 gap-4">
          {/* Rust */}
          <div className="p-4 rounded-xl bg-orange-500/10 border border-orange-500/20">
            <div className="flex items-center gap-3 mb-2">
              <div className="w-8 h-8 rounded-lg bg-orange-500/20 flex items-center justify-center">
                <svg className="w-4 h-4 text-orange-400" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M23.8 14.3l-1.3-.8c0-.2.1-.4.1-.6 0-.2 0-.4-.1-.6l1.3-.8c.1-.1.2-.3.1-.4l-1.2-2c-.1-.2-.3-.2-.4-.2l-1.5.5c-.3-.2-.7-.4-1-.6l-.2-1.6c0-.2-.2-.3-.3-.3h-2.4c-.2 0-.3.1-.3.3l-.2 1.6c-.4.2-.7.4-1 .6l-1.5-.5c-.2-.1-.4 0-.4.2l-1.2 2c-.1.1 0 .3.1.4l1.3.8c0 .2-.1.4-.1.6 0 .2 0 .4.1.6l-1.3.8c-.1.1-.2.3-.1.4l1.2 2c.1.2.3.2.4.2l1.5-.5c.3.2.7.4 1 .6l.2 1.6c0 .2.2.3.3.3h2.4c.2 0 .3-.1.3-.3l.2-1.6c.4-.2.7-.4 1-.6l1.5.5c.2.1.4 0 .4-.2l1.2-2c.1-.1 0-.3-.1-.4zM19 14.5c-.8 0-1.5-.7-1.5-1.5s.7-1.5 1.5-1.5 1.5.7 1.5 1.5-.7 1.5-1.5 1.5z"/>
                </svg>
              </div>
              <h4 className="font-semibold text-white">{t.rustPowered}</h4>
            </div>
            <p className="text-sm text-zinc-500">{t.rustPoweredDesc}</p>
          </div>

          {/* pgvector */}
          <div className="p-4 rounded-xl bg-blue-500/10 border border-blue-500/20">
            <div className="flex items-center gap-3 mb-2">
              <div className="w-8 h-8 rounded-lg bg-blue-500/20 flex items-center justify-center">
                <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                </svg>
              </div>
              <h4 className="font-semibold text-white">{t.pgvectorIntegration}</h4>
            </div>
            <p className="text-sm text-zinc-500">{t.pgvectorDesc}</p>
          </div>

          {/* HNSW */}
          <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <div className="flex items-center gap-3 mb-2">
              <div className="w-8 h-8 rounded-lg bg-emerald-500/20 flex items-center justify-center">
                <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              <h4 className="font-semibold text-white">{t.hnswIndex}</h4>
            </div>
            <p className="text-sm text-zinc-500">{t.hnswDesc}</p>
          </div>
        </div>
      </section>
    </div>
  );
}

function DataTab({ liveStats, t }: DataTabProps) {
  return (
    <div className="space-y-12">
      {/* Data Source */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
            </svg>
          </span>
          {t.dataSourceTVDB}
          <span className="relative flex h-2 w-2 ml-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          {t.dataSourceText}
        </p>
        <div className="grid md:grid-cols-4 gap-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-blue-500/30 transition-all">
            <div className="text-2xl font-bold text-white mb-1 transition-transform group-hover:scale-110">
              {liveStats?.totalSeries?.toLocaleString() || '1,176'}
            </div>
            <div className="text-zinc-500 text-sm">{t.tvSeries}</div>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-purple-500/30 transition-all">
            <div className="text-2xl font-bold text-white mb-1 transition-transform group-hover:scale-110">
              {liveStats?.totalMovies?.toLocaleString() || '1,039'}
            </div>
            <div className="text-zinc-500 text-sm">{t.movies}</div>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-emerald-500/30 transition-all">
            <div className="text-2xl font-bold text-emerald-400 mb-1">100%</div>
            <div className="text-zinc-500 text-sm">{t.withEmbeddings}</div>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-orange-500/30 transition-all">
            <div className="text-2xl font-bold text-white mb-1">20+</div>
            <div className="text-zinc-500 text-sm">{t.languages}</div>
          </div>
        </div>
      </section>

      {/* Vector Embeddings */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
            </svg>
          </span>
          {t.vectorEmbeddingsExplained}
        </h2>
        <p className="text-zinc-400 text-lg mb-6" dangerouslySetInnerHTML={{ __html: t.vectorEmbeddingsExplainedText }} />

        <div className="bg-zinc-900 rounded-xl p-6 mb-6 font-mono text-sm overflow-x-auto">
          <div className="text-zinc-500 mb-2">// Example: "The Batman" might have a vector like:</div>
          <div className="text-emerald-400">[0.234, -0.891, 0.456, 0.123, ... 380 more numbers]</div>
          <div className="text-zinc-500 mt-4 mb-2">// Similar movies have similar vectors:</div>
          <div className="text-blue-400">"The Dark Knight" → [0.241, -0.887, 0.448, 0.131, ...]</div>
          <div className="text-zinc-500 mt-1">// Distance: 0.02 (very similar!)</div>
        </div>

        <h3 className="text-lg font-semibold text-white mb-3">{t.why384Dimensions}</h3>
        <p className="text-zinc-400 mb-4">
          {t.why384Text}
        </p>
        <ul className="grid md:grid-cols-2 gap-2 text-zinc-400">
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            {t.genreCombinations}
          </li>
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            {t.toneAndMood}
          </li>
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            {t.eraAndStyle}
          </li>
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            {t.thematicElements}
          </li>
        </ul>
      </section>

      {/* RuVector */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </span>
          {t.ruVectorTitle}
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          {t.ruVectorText}
        </p>

        <div className="grid md:grid-cols-2 gap-6">
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">{t.distanceFunctions}</h3>
            <div className="space-y-3">
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="text-white font-medium">{t.cosineDistance}</div>
                <div className="text-zinc-500 text-sm">{t.cosineDistanceDesc}</div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="text-white font-medium">{t.euclideanDistance}</div>
                <div className="text-zinc-500 text-sm">{t.euclideanDistanceDesc}</div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-emerald-500/30">
                <div className="text-emerald-400 font-medium">{t.hyperbolicDistance} ✨</div>
                <div className="text-zinc-500 text-sm">{t.hyperbolicDistanceDesc}</div>
              </div>
            </div>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white mb-3 flex items-center gap-2">
              {t.performance}
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
            </h3>
            <div className="space-y-3">
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-emerald-500/30">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">{t.currentQueryLatency}</span>
                  <span className="text-emerald-400 font-mono font-bold">{liveStats?.avgLatency || '~3ms'}</span>
                </div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">{t.vectorSearch}</span>
                  <span className="text-emerald-400 font-mono">~7.8ms</span>
                </div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">{t.simdAcceleration}</span>
                  <span className="text-emerald-400">{t.enabled}</span>
                </div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">{t.learnedPatterns}</span>
                  <span className="text-emerald-400">{liveStats?.totalPatterns || 0} {t.activeLabel}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

interface ComparisonTabProps {
  t: typeof translations.en;
}

function ComparisonTab({ t }: ComparisonTabProps) {
  return (
    <div className="space-y-12">
      <section>
        <h2 className="text-2xl font-bold text-white mb-6">{t.traditionalVsOur}</h2>

        {/* Comparison Table */}
        <div className="glass-card rounded-2xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-zinc-700">
                  <th className="text-left p-4 text-zinc-400 font-medium">{t.aspect}</th>
                  <th className="text-left p-4 text-zinc-400 font-medium">{t.traditionalSystems}</th>
                  <th className="text-left p-4 text-emerald-400 font-medium">{t.ourSystem}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800">
                <tr>
                  <td className="p-4 text-white font-medium">{t.learningSpeed}</td>
                  <td className="p-4 text-zinc-400">{t.batchUpdates}</td>
                  <td className="p-4 text-emerald-400">{t.realTimeEveryInteraction}</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">{t.coldStartProblem}</td>
                  <td className="p-4 text-zinc-400">{t.needsExtensiveHistory}</td>
                  <td className="p-4 text-emerald-400">{t.worksFromFirstClick}</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">{t.similarityMatching}</td>
                  <td className="p-4 text-zinc-400">{t.genreTagMatching}</td>
                  <td className="p-4 text-emerald-400">{t.semanticVectors}</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">{t.feedbackIntegration}</td>
                  <td className="p-4 text-zinc-400">{t.explicitRatingsOnly}</td>
                  <td className="p-4 text-emerald-400">{t.watchSkipImplicit}</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">{t.searchCapability}</td>
                  <td className="p-4 text-zinc-400">{t.keywordMatching}</td>
                  <td className="p-4 text-emerald-400">{t.naturalLanguageUnderstanding}</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">{t.exploration}</td>
                  <td className="p-4 text-zinc-400">{t.fixedRecommendations}</td>
                  <td className="p-4 text-emerald-400">{t.epsilonGreedy}</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">{t.hierarchyUnderstanding}</td>
                  <td className="p-4 text-zinc-400">{t.flatCategories}</td>
                  <td className="p-4 text-emerald-400">{t.hyperbolicGeometry}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Visual Comparison */}
      <section className="grid md:grid-cols-2 gap-6">
        <div className="glass-card rounded-2xl p-6 border-red-500/20">
          <h3 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
            <span className="w-8 h-8 rounded-lg bg-red-500/20 flex items-center justify-center">
              <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </span>
            {t.traditionalApproach}
          </h3>
          <div className="space-y-3 text-zinc-400">
            <p>1. User watches "The Office"</p>
            <p>2. System looks up "Comedy" genre</p>
            <p>3. Returns all comedies alphabetically</p>
            <p>4. No learning from skipped shows</p>
            <p>5. Same recommendations forever</p>
          </div>
          <div className="mt-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
            {t.traditionalResult}
          </div>
        </div>

        <div className="glass-card rounded-2xl p-6 border-emerald-500/20">
          <h3 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
            <span className="w-8 h-8 rounded-lg bg-emerald-500/20 flex items-center justify-center">
              <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </span>
            {t.ourApproachLabel}
          </h3>
          <div className="space-y-3 text-zinc-400">
            <p>1. User watches "The Office"</p>
            <p>2. Vector finds semantically similar shows</p>
            <p>3. Q-Learning ranks by pattern success</p>
            <p>4. User skips "Parks & Rec" → learns preference</p>
            <p>5. Recommendations evolve with each click</p>
          </div>
          <div className="mt-4 p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm">
            {t.ourResult}
          </div>
        </div>
      </section>

      {/* Why It's Better */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">{t.whyQLearningWins}</h2>
        <div className="grid md:grid-cols-2 gap-8">
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">{t.mathBehindIt}</h3>
            <div className="bg-zinc-900 rounded-xl p-4 font-mono text-sm">
              <div className="text-zinc-500 mb-2">// Q-Learning Update Formula</div>
              <div className="text-emerald-400">Q(s,a) ← Q(s,a) + α[r + γ·max(Q(s',a')) - Q(s,a)]</div>
              <div className="text-zinc-500 mt-4 text-xs">
                <div>α = learning rate (0.1)</div>
                <div>γ = discount factor (0.95)</div>
                <div>r = reward (+0.5 to +1 for watch, -0.3 to -0.1 for skip)</div>
              </div>
            </div>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">{t.inPlainEnglish}</h3>
            <p className="text-zinc-400">
              {t.inPlainEnglishText}
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}

interface ArchitectureTabProps {
  t: typeof translations.en;
}

function ArchitectureTab({ t }: ArchitectureTabProps) {
  return (
    <div className="space-y-12">
      {/* System Architecture Diagram */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">{t.systemArchitecture}</h2>

        {/* Mermaid-style diagram using divs */}
        <div className="bg-zinc-900 rounded-xl p-8 overflow-x-auto">
          <div className="min-w-[600px]">
            {/* Top Layer - UI */}
            <div className="flex justify-center mb-8">
              <div className="px-6 py-3 rounded-xl bg-blue-500/20 border border-blue-500/30 text-blue-400 font-medium">
                Next.js Frontend (React)
              </div>
            </div>

            {/* Arrow */}
            <div className="flex justify-center mb-8">
              <div className="flex flex-col items-center text-zinc-600">
                <div className="w-0.5 h-8 bg-zinc-700" />
                <div className="text-xs">HTTP/API</div>
                <div className="w-0.5 h-8 bg-zinc-700" />
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
            </div>

            {/* Middle Layer - API Routes */}
            <div className="flex justify-center gap-4 mb-8 flex-wrap">
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/recommendations
              </div>
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/similar
              </div>
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/feedback
              </div>
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/search
              </div>
            </div>

            {/* Arrow */}
            <div className="flex justify-center mb-8">
              <div className="flex flex-col items-center text-zinc-600">
                <div className="w-0.5 h-8 bg-zinc-700" />
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
            </div>

            {/* Core Services */}
            <div className="flex justify-center gap-6 mb-8">
              <div className="p-4 rounded-xl bg-emerald-500/20 border border-emerald-500/30 text-center">
                <div className="text-emerald-400 font-medium mb-1">Q-Learning Engine</div>
                <div className="text-zinc-500 text-xs">ruvector-learning.ts</div>
              </div>
              <div className="p-4 rounded-xl bg-orange-500/20 border border-orange-500/30 text-center">
                <div className="text-orange-400 font-medium mb-1">RuVector</div>
                <div className="text-zinc-500 text-xs">PostgreSQL Extension</div>
              </div>
            </div>

            {/* Arrow */}
            <div className="flex justify-center mb-8">
              <div className="flex flex-col items-center text-zinc-600">
                <div className="w-0.5 h-8 bg-zinc-700" />
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
            </div>

            {/* Database Layer */}
            <div className="flex justify-center gap-6">
              <div className="p-4 rounded-xl bg-zinc-700/50 border border-zinc-600 text-center">
                <div className="text-white font-medium mb-1">PostgreSQL</div>
                <div className="text-zinc-500 text-xs">Content + Vectors + Patterns</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Data Flow */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">{t.dataFlowWatch}</h2>

        <div className="space-y-4">
          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center text-blue-400 font-bold flex-shrink-0">1</div>
            <div className="flex-1">
              <div className="text-white font-medium">User clicks "Watch" on content</div>
              <div className="text-zinc-500 text-sm">Frontend sends POST to /api/feedback</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">ContentCard.tsx</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-400 font-bold flex-shrink-0">2</div>
            <div className="flex-1">
              <div className="text-white font-medium">API calculates reward</div>
              <div className="text-zinc-500 text-sm">Positive reward: +0.5 to +1.0 (with exploration noise)</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">feedback/route.ts</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center text-emerald-400 font-bold flex-shrink-0">3</div>
            <div className="flex-1">
              <div className="text-white font-medium">Pattern identified from content</div>
              <div className="text-zinc-500 text-sm">e.g., "action_thriller" or "mood_exciting"</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">ruvector-learning.ts</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-orange-500/20 flex items-center justify-center text-orange-400 font-bold flex-shrink-0">4</div>
            <div className="flex-1">
              <div className="text-white font-medium">Q-Learning updates pattern stats</div>
              <div className="text-zinc-500 text-sm">Success rate and avg reward updated in DB</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">recommendation_patterns</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-yellow-500/20 flex items-center justify-center text-yellow-400 font-bold flex-shrink-0">5</div>
            <div className="flex-1">
              <div className="text-white font-medium">Feedback recorded for history</div>
              <div className="text-zinc-500 text-sm">Stored in learning_feedback table</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">learning_feedback</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <div className="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center text-emerald-400 font-bold flex-shrink-0">✓</div>
            <div className="flex-1">
              <div className="text-emerald-400 font-medium">Future recommendations improved</div>
              <div className="text-zinc-500 text-sm">Top patterns boost similar content in next query</div>
            </div>
          </div>
        </div>
      </section>

      {/* Database Schema */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">{t.keyDatabaseTables}</h2>

        <div className="grid md:grid-cols-2 gap-6">
          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-emerald-400 font-mono font-medium mb-3">content</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="flex justify-between"><span className="text-zinc-400">id</span><span className="text-zinc-600">UUID</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">title</span><span className="text-zinc-600">TEXT</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">genres</span><span className="text-zinc-600">TEXT[]</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">embedding</span><span className="text-purple-400">VECTOR(384)</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">image_url</span><span className="text-zinc-600">TEXT</span></div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-emerald-400 font-mono font-medium mb-3">recommendation_patterns</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="flex justify-between"><span className="text-zinc-400">pattern_type</span><span className="text-zinc-600">TEXT</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">success_rate</span><span className="text-emerald-400">DECIMAL</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">total_uses</span><span className="text-zinc-600">INTEGER</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">avg_reward</span><span className="text-emerald-400">DECIMAL</span></div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-emerald-400 font-mono font-medium mb-3">learning_feedback</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="flex justify-between"><span className="text-zinc-400">content_id</span><span className="text-zinc-600">UUID</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">pattern_id</span><span className="text-zinc-600">INTEGER</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">was_successful</span><span className="text-zinc-600">BOOLEAN</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">reward</span><span className="text-emerald-400">DECIMAL</span></div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-orange-400 font-mono font-medium mb-3">RuVector Functions</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="text-zinc-400">ruvector_cosine_distance()</div>
              <div className="text-zinc-400">ruvector_euclidean_distance()</div>
              <div className="text-purple-400">ruvector_hyperbolic_distance()</div>
              <div className="text-zinc-400">ruvector_learn_from_feedback()</div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

interface ResearchTabProps {
  t: typeof translations.en;
}

function ResearchTab({ t }: ResearchTabProps) {
  return (
    <div className="space-y-12">
      {/* Research Background */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          </span>
          {t.researchFoundation}
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          {t.researchFoundationText}
        </p>

        <div className="space-y-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <h3 className="text-white font-semibold mb-2">Q-Learning (Watkins, 1989)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              A model-free reinforcement learning algorithm that learns the value of actions in states.
              We use it to learn which recommendation patterns are most successful.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              "Learning from Delayed Rewards" - Christopher Watkins, PhD Thesis
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <h3 className="text-white font-semibold mb-2">Sentence-BERT Embeddings (2019)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Dense vector representations that capture semantic meaning of text.
              We use 384-dimensional embeddings to represent content descriptions.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Reimers & Gurevych - "Sentence-BERT: Sentence Embeddings using Siamese BERT-Networks"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Hyperbolic Embeddings (Nickel & Kiela, 2017)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Embeddings in hyperbolic space better represent hierarchical relationships.
              Perfect for genre → subgenre → content hierarchies in media.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              "Poincaré Embeddings for Learning Hierarchical Representations" - Facebook AI Research
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Thompson Sampling (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Bayesian approach using Beta distributions for each arm. Samples from posterior
              to balance exploration/exploitation naturally without ε parameter.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Thompson (1933) - "On the Likelihood that One Unknown Probability Exceeds Another"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">UCB1 Algorithm (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Upper Confidence Bound with optimistic estimation. Balances average reward with
              exploration bonus based on uncertainty.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Auer et al. (2002) - "Finite-time Analysis of the Multiarmed Bandit Problem"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">LinUCB Contextual Bandits (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Linear contextual bandits that use content features (genre, mood, time)
              to make context-aware recommendations.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Li et al. (2010) - "A Contextual-Bandit Approach to Personalized News Article Recommendation"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Double Q-Learning (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Uses two Q-tables to decouple action selection from evaluation,
              preventing overestimation bias in value estimates.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              van Hasselt (2010) - "Double Q-learning"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Prioritized Experience Replay (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Samples experiences proportional to their TD-error magnitude.
              Learns faster from surprising or important transitions.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Schaul et al. (2015) - "Prioritized Experience Replay" - DeepMind
            </div>
          </div>
        </div>
      </section>

      {/* Optimizations */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">{t.optimizationsInnovations}</h2>

        <div className="grid md:grid-cols-2 gap-6">
          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">⚡</div>
            <h3 className="text-white font-semibold mb-2">SIMD Vector Operations</h3>
            <p className="text-zinc-400 text-sm">
              RuVector uses CPU SIMD instructions to compute distance between 384-dimensional
              vectors in microseconds. This is 10-100x faster than naive implementations.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">🔄</div>
            <h3 className="text-white font-semibold mb-2">Pattern Boosting</h3>
            <p className="text-zinc-400 text-sm">
              Instead of re-ranking all results, we boost queries with top-performing patterns.
              This maintains database query efficiency while incorporating learned preferences.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">📊</div>
            <h3 className="text-white font-semibold mb-2">Incremental Learning</h3>
            <p className="text-zinc-400 text-sm">
              Pattern statistics are updated incrementally with each feedback.
              No need to retrain or rebuild indices - learning happens in milliseconds.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">🎯</div>
            <h3 className="text-white font-semibold mb-2">Mood-to-Genre Mapping</h3>
            <p className="text-zinc-400 text-sm">
              Pre-computed mappings from emotional moods to genre combinations enable
              instant filtering without complex NLP at query time.
            </p>
          </div>
        </div>
      </section>

      {/* Future Work */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">{t.futureDirections}</h2>

        <div className="space-y-4">
          <div className="flex items-start gap-4 p-4 rounded-xl bg-zinc-800/30">
            <div className="w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-purple-400 font-bold text-sm">1</span>
            </div>
            <div>
              <h3 className="text-white font-medium">Multi-User Collaborative Filtering</h3>
              <p className="text-zinc-500 text-sm">
                Combine individual Q-Learning with collaborative signals from similar users
                for even better cold-start performance.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <div className="w-8 h-8 rounded-lg bg-emerald-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-emerald-400 font-bold text-sm">✓</span>
            </div>
            <div>
              <h3 className="text-emerald-400 font-medium">Contextual Bandits (IMPLEMENTED)</h3>
              <p className="text-zinc-500 text-sm">
                Thompson Sampling, UCB1, and LinUCB now implemented for efficient exploration-exploitation.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 p-4 rounded-xl bg-zinc-800/30">
            <div className="w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-purple-400 font-bold text-sm">3</span>
            </div>
            <div>
              <h3 className="text-white font-medium">Time-Aware Recommendations</h3>
              <p className="text-zinc-500 text-sm">
                Learn patterns based on time of day, day of week, and seasonal preferences.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 p-4 rounded-xl bg-zinc-800/30">
            <div className="w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-purple-400 font-bold text-sm">4</span>
            </div>
            <div>
              <h3 className="text-white font-medium">Cross-Modal Learning</h3>
              <p className="text-zinc-500 text-sm">
                Incorporate poster image embeddings alongside text for richer content representation.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Tech Stack */}
      <section>
        <h2 className="text-2xl font-bold text-white mb-6">{t.technologyStack}</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {[
            { name: 'Next.js 14', desc: 'React Framework' },
            { name: 'TypeScript', desc: 'Type Safety' },
            { name: 'PostgreSQL', desc: 'Database' },
            { name: 'RuVector', desc: 'Vector Extension' },
            { name: 'Tailwind CSS', desc: 'Styling' },
            { name: 'TVDB API', desc: 'Content Data' },
            { name: 'Rust', desc: 'RuVector Core' },
            { name: 'Vercel', desc: 'Deployment' },
          ].map((tech, i) => (
            <div key={i} className="glass-card rounded-xl p-4 text-center hover:border-emerald-500/30 transition-colors">
              <div className="text-white font-medium">{tech.name}</div>
              <div className="text-zinc-500 text-sm">{tech.desc}</div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
