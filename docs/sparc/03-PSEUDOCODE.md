# SPARC Pseudocode: Obsidian Clone in Rust

**Project:** ObsidianRS - A Backward-Compatible Obsidian Clone in Rust
**Version:** 1.0.0
**Date:** 2025-12-07
**Status:** Pseudocode Phase Complete

---

## 1. Core Data Structures

### 1.1 Note and Vault Structures

```pseudocode
// Core note representation
STRUCTURE Note:
    id: String              // Unique identifier (relative path)
    path: Path              // Absolute file path
    basename: String        // Filename without extension
    extension: String       // File extension (.md)
    content: String         // Raw markdown content
    frontmatter: Frontmatter OR NULL
    stat: FileStat

STRUCTURE Frontmatter:
    title: String OR NULL
    tags: List<String>
    aliases: List<String>
    created: DateTime OR NULL
    modified: DateTime OR NULL
    cssclasses: List<String>
    extra: Map<String, Any>     // Additional YAML fields

STRUCTURE FileStat:
    created: DateTime
    modified: DateTime
    size: Integer               // Bytes

// Cached metadata for fast queries
STRUCTURE CachedMetadata:
    links: List<LinkCache>
    embeds: List<EmbedCache>
    tags: List<TagCache>
    headings: List<HeadingCache>
    blocks: Map<String, BlockCache>
    frontmatter: Frontmatter OR NULL

STRUCTURE LinkCache:
    link: String                // Target note name
    original: String            // Original markdown [[...]]
    display_text: String OR NULL
    heading: String OR NULL     // #heading
    block_id: String OR NULL    // ^block-id
    position: Position

STRUCTURE Position:
    start: Location
    end: Location

STRUCTURE Location:
    line: Integer
    column: Integer
    offset: Integer             // Byte offset in file
```

### 1.2 Knowledge Graph Structures

```pseudocode
// Graph node representing a note
STRUCTURE NoteNode:
    id: String
    title: String
    tags: List<String>
    link_count: Integer         // Total incoming + outgoing

// Graph edge representing a link
ENUMERATION LinkType:
    REFERENCE                   // [[link]]
    EMBED                       // ![[embed]]
    TAG                         // Shared tag connection

STRUCTURE LinkEdge:
    link_type: LinkType
    weight: Float               // For weighted algorithms

// Main knowledge graph
STRUCTURE KnowledgeGraph:
    nodes: Map<String, NodeIndex>
    graph: DirectedGraph<NoteNode, LinkEdge>
    resolved_links: Map<String, Map<String, Integer>>    // source -> target -> count
    unresolved_links: Map<String, Map<String, Integer>>  // source -> missing -> count
```

---

## 2. Vault Operations

### 2.1 Vault Initialization

```pseudocode
FUNCTION initialize_vault(vault_path: Path) -> Result<Vault>:
    // Validate path exists
    IF NOT exists(vault_path):
        RETURN Error("Vault path does not exist")

    // Create .obsidian directory if needed
    obsidian_dir = vault_path / ".obsidian"
    IF NOT exists(obsidian_dir):
        create_directory(obsidian_dir)
        create_default_config(obsidian_dir)

    // Initialize storage components
    vector_db = RuVector.new(config: {
        dimensions: 384,
        metric: COSINE,
        m: 32,
        ef_construction: 200
    })

    fulltext_index = Tantivy.create_index(
        schema: create_notes_schema(),
        path: obsidian_dir / "search"
    )

    metadata_store = Redb.open(obsidian_dir / "metadata.db")

    knowledge_graph = KnowledgeGraph.new()

    // Scan and index all notes
    notes = scan_vault(vault_path)
    FOR EACH note IN notes PARALLEL:
        index_note(note, vector_db, fulltext_index, metadata_store, knowledge_graph)

    // Start file watcher
    watcher = FileWatcher.new(vault_path)
    watcher.on_change(handle_file_change)

    RETURN Vault {
        path: vault_path,
        vector_db,
        fulltext_index,
        metadata_store,
        knowledge_graph,
        watcher
    }
```

### 2.2 Note CRUD Operations

```pseudocode
FUNCTION create_note(vault: Vault, path: String, content: String) -> Result<Note>:
    full_path = vault.path / path

    // Validate path
    IF exists(full_path):
        RETURN Error("Note already exists")

    // Create parent directories if needed
    parent = parent_directory(full_path)
    IF NOT exists(parent):
        create_directory_recursive(parent)

    // Write file
    write_file(full_path, content)

    // Get file stats
    stat = get_file_stat(full_path)

    // Parse note
    parsed = parse_markdown(content)

    // Create note object
    note = Note {
        id: path,
        path: full_path,
        basename: stem(path),
        extension: extension(path),
        content,
        frontmatter: parsed.frontmatter,
        stat
    }

    // Index note (async)
    SPAWN index_note_async(note, vault)

    // Emit event
    vault.events.emit("create", note)

    RETURN note


FUNCTION read_note(vault: Vault, path: String) -> Result<String>:
    full_path = vault.path / path

    IF NOT exists(full_path):
        RETURN Error("Note not found")

    content = read_file(full_path)
    RETURN content


FUNCTION modify_note(vault: Vault, note: Note, new_content: String) -> Result<()>:
    // Write new content
    write_file(note.path, new_content)

    // Update note object
    note.content = new_content
    note.stat.modified = now()

    // Re-parse and update cache
    parsed = parse_markdown(new_content)
    note.frontmatter = parsed.frontmatter

    // Update indexes (async)
    SPAWN reindex_note_async(note, vault)

    // Emit event
    vault.events.emit("modify", note)

    RETURN Ok


FUNCTION delete_note(vault: Vault, note: Note, use_trash: Boolean) -> Result<()>:
    IF use_trash:
        trash_path = vault.path / ".trash" / note.id
        move_file(note.path, trash_path)
    ELSE:
        delete_file(note.path)

    // Remove from indexes
    vault.vector_db.delete(note.id)
    vault.fulltext_index.delete(note.id)
    vault.metadata_store.delete(note.id)
    vault.knowledge_graph.remove_node(note.id)

    // Update backlinks (remove references to deleted note)
    FOR EACH source IN get_backlinks(vault, note.id):
        update_unresolved_links(vault, source, note.id)

    // Emit event
    vault.events.emit("delete", note)

    RETURN Ok


FUNCTION rename_note(vault: Vault, note: Note, new_path: String) -> Result<()>:
    old_path = note.path
    old_id = note.id
    new_full_path = vault.path / new_path

    // Check new path doesn't exist
    IF exists(new_full_path):
        RETURN Error("Target path already exists")

    // Move file
    move_file(old_path, new_full_path)

    // Update note object
    note.path = new_full_path
    note.id = new_path
    note.basename = stem(new_path)

    // Update all links pointing to this note
    backlinks = get_backlinks(vault, old_id)
    FOR EACH source_note IN backlinks:
        update_links_in_note(vault, source_note, old_id, note.id)

    // Update indexes with new ID
    vault.vector_db.update_id(old_id, note.id)
    vault.fulltext_index.update_id(old_id, note.id)
    vault.metadata_store.update_id(old_id, note.id)
    vault.knowledge_graph.rename_node(old_id, note.id)

    // Emit event
    vault.events.emit("rename", note, old_path)

    RETURN Ok
```

---

## 3. Markdown Processing

### 3.1 Markdown Parser

```pseudocode
STRUCTURE ParsedMarkdown:
    content: String             // Content without frontmatter
    frontmatter: Frontmatter OR NULL
    links: List<WikiLink>
    embeds: List<Embed>
    tags: List<Tag>
    headings: List<Heading>
    code_blocks: List<CodeBlock>

FUNCTION parse_markdown(raw: String) -> ParsedMarkdown:
    result = ParsedMarkdown.new()

    // Extract frontmatter
    IF raw.starts_with("---\n"):
        end_pos = raw.find("\n---\n", start: 4)
        IF end_pos != NULL:
            yaml_content = raw[4..end_pos]
            result.frontmatter = parse_yaml(yaml_content)
            result.content = raw[end_pos + 5..]
        ELSE:
            result.content = raw
    ELSE:
        result.content = raw

    // Parse wikilinks: [[target|display]]
    link_regex = /\[\[([^\]]+)\]\]/
    FOR EACH match IN link_regex.find_all(result.content):
        link = parse_wikilink(match)
        result.links.push(link)

    // Parse embeds: ![[target]]
    embed_regex = /!\[\[([^\]]+)\]\]/
    FOR EACH match IN embed_regex.find_all(result.content):
        embed = parse_embed(match)
        result.embeds.push(embed)

    // Parse tags: #tag or #nested/tag
    tag_regex = /#([a-zA-Z0-9_\/-]+)/
    FOR EACH match IN tag_regex.find_all(result.content):
        tag = Tag {
            name: match.group(1),
            position: match.position()
        }
        result.tags.push(tag)

    // Parse headings using pulldown-cmark events
    parser = PulldownCmark.new(result.content)
    FOR EACH event IN parser:
        IF event IS HeadingStart(level):
            heading_text = collect_until(HeadingEnd)
            result.headings.push(Heading {
                text: heading_text,
                level: level,
                position: event.position()
            })

    RETURN result


FUNCTION parse_wikilink(match: RegexMatch) -> WikiLink:
    content = match.group(1)

    // Check for display text: [[target|display]]
    IF content.contains("|"):
        parts = content.split("|")
        target = parts[0]
        display = parts[1]
    ELSE:
        target = content
        display = NULL

    // Check for heading: target#heading
    IF target.contains("#"):
        parts = target.split("#", limit: 2)
        note_name = parts[0]
        subpath = parts[1]

        // Check for block reference: #^block-id
        IF subpath.starts_with("^"):
            block_id = subpath[1..]
            heading = NULL
        ELSE:
            heading = subpath
            block_id = NULL
    ELSE:
        note_name = target
        heading = NULL
        block_id = NULL

    RETURN WikiLink {
        target: note_name,
        display: display,
        heading: heading,
        block_id: block_id,
        position: match.position()
    }
```

### 3.2 Markdown Renderer

```pseudocode
FUNCTION render_to_html(markdown: String, vault: Vault) -> String:
    // First parse for custom syntax
    parsed = parse_markdown(markdown)

    // Pre-process custom syntax before pulldown-cmark
    processed = preprocess_obsidian_syntax(parsed.content, vault)

    // Use pulldown-cmark for standard markdown
    options = ENABLE_TABLES | ENABLE_FOOTNOTES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS
    parser = PulldownCmark.new_ext(processed, options)

    // Transform events for custom rendering
    events = []
    FOR EACH event IN parser:
        MATCH event:
            CASE Text(text):
                // Handle highlights ==text==
                IF text.contains("=="):
                    transformed = transform_highlights(text)
                    events.push(Html(transformed))
                ELSE:
                    events.push(event)

            CASE Code(code):
                // Syntax highlighting
                highlighted = highlight_code(code.content, code.language)
                events.push(Html(highlighted))

            DEFAULT:
                events.push(event)

    // Convert to HTML
    html = pulldown_cmark_to_html(events)

    RETURN html


FUNCTION preprocess_obsidian_syntax(content: String, vault: Vault) -> String:
    result = content

    // Transform wikilinks to HTML links
    wikilink_regex = /\[\[([^\]]+)\]\]/
    result = wikilink_regex.replace_all(result, (match) ->
        link = parse_wikilink(match)
        resolved = resolve_link(vault, link.target)

        display = link.display OR link.target

        IF resolved != NULL:
            href = "/note/" + url_encode(resolved.id)
            class = "internal-link"
        ELSE:
            href = "#"
            class = "internal-link is-unresolved"

        RETURN "<a href='{href}' class='{class}'>{display}</a>"
    )

    // Transform embeds to appropriate HTML
    embed_regex = /!\[\[([^\]]+)\]\]/
    result = embed_regex.replace_all(result, (match) ->
        target = match.group(1)

        IF is_image(target):
            src = resolve_attachment(vault, target)
            RETURN "<img src='{src}' alt='{target}' class='internal-embed'/>"
        ELSE IF is_pdf(target):
            src = resolve_attachment(vault, target)
            RETURN "<embed src='{src}' type='application/pdf' class='internal-embed'/>"
        ELSE:
            // Embed note content
            note = resolve_link(vault, target)
            IF note != NULL:
                embedded_html = render_to_html(note.content, vault)
                RETURN "<div class='embedded-note'>{embedded_html}</div>"
            ELSE:
                RETURN "<span class='embed-unresolved'>![[{target}]]</span>"
    )

    // Transform tags to clickable elements
    tag_regex = /#([a-zA-Z0-9_\/-]+)/
    result = tag_regex.replace_all(result, (match) ->
        tag = match.group(1)
        RETURN "<a href='/tag/{tag}' class='tag'>#{tag}</a>"
    )

    RETURN result
```

---

## 4. Search Engine

### 4.1 Hybrid Search Implementation

```pseudocode
STRUCTURE SearchResult:
    note_id: String
    score: Float
    matches: List<TextMatch>
    snippet: String

STRUCTURE TextMatch:
    text: String
    position: Position
    match_type: MatchType      // TITLE, CONTENT, TAG, FRONTMATTER

FUNCTION hybrid_search(vault: Vault, query: String, limit: Integer) -> List<SearchResult>:
    // Execute searches in parallel
    PARALLEL:
        keyword_results = keyword_search(vault.fulltext_index, query, limit * 2)
        semantic_results = semantic_search(vault.vector_db, query, limit * 2)

    // Merge results using reciprocal rank fusion
    merged = merge_search_results(
        keyword_results,
        semantic_results,
        weights: { keyword: 0.4, semantic: 0.6 }
    )

    // Generate snippets for top results
    FOR EACH result IN merged[0..limit]:
        result.snippet = generate_snippet(vault, result.note_id, query)

    RETURN merged[0..limit]


FUNCTION keyword_search(index: TantivyIndex, query: String, limit: Integer) -> List<SearchResult>:
    // Parse query for operators
    parsed_query = parse_search_query(query)

    // Build tantivy query
    tantivy_query = QueryBuilder.new()

    IF parsed_query.has_tag_filter:
        tantivy_query.add_term("tags", parsed_query.tag)

    IF parsed_query.has_path_filter:
        tantivy_query.add_prefix("path", parsed_query.path)

    IF parsed_query.has_content:
        tantivy_query.add_fuzzy("content", parsed_query.content, distance: 2)
        tantivy_query.add_fuzzy("title", parsed_query.content, distance: 1, boost: 2.0)

    // Execute search
    searcher = index.reader().searcher()
    top_docs = searcher.search(tantivy_query.build(), limit)

    results = []
    FOR EACH (score, doc_address) IN top_docs:
        doc = searcher.doc(doc_address)
        results.push(SearchResult {
            note_id: doc.get("id"),
            score: score,
            matches: extract_matches(doc, query)
        })

    RETURN results


FUNCTION semantic_search(vector_db: RuVector, query: String, limit: Integer) -> List<SearchResult>:
    // Generate query embedding
    query_embedding = generate_embedding(query)

    // Search vector database
    neighbors = vector_db.search(query_embedding, k: limit)

    results = []
    FOR EACH (id, distance) IN neighbors:
        // Convert distance to similarity score
        score = 1.0 - distance  // For cosine distance
        results.push(SearchResult {
            note_id: id,
            score: score,
            matches: []  // Semantic search doesn't have text matches
        })

    RETURN results


FUNCTION merge_search_results(
    keyword: List<SearchResult>,
    semantic: List<SearchResult>,
    weights: Map<String, Float>
) -> List<SearchResult>:

    // Use Reciprocal Rank Fusion (RRF)
    k = 60  // RRF constant

    scores = Map<String, Float>.new()
    results_map = Map<String, SearchResult>.new()

    // Process keyword results
    FOR EACH (rank, result) IN enumerate(keyword):
        rrf_score = weights.keyword / (k + rank + 1)
        scores[result.note_id] = scores.get(result.note_id, 0) + rrf_score
        results_map[result.note_id] = result

    // Process semantic results
    FOR EACH (rank, result) IN enumerate(semantic):
        rrf_score = weights.semantic / (k + rank + 1)
        scores[result.note_id] = scores.get(result.note_id, 0) + rrf_score
        IF result.note_id NOT IN results_map:
            results_map[result.note_id] = result

    // Sort by combined score
    sorted_ids = sort_by_value_descending(scores)

    merged = []
    FOR EACH id IN sorted_ids:
        result = results_map[id]
        result.score = scores[id]
        merged.push(result)

    RETURN merged
```

### 4.2 Search Indexing

```pseudocode
FUNCTION index_note(note: Note, vault: Vault) -> Result<()>:
    // Parse markdown
    parsed = parse_markdown(note.content)

    // Index in full-text search (tantivy)
    index_fulltext(vault.fulltext_index, note, parsed)

    // Index in vector database (ruvector)
    index_vectors(vault.vector_db, note, parsed)

    // Store metadata
    store_metadata(vault.metadata_store, note, parsed)

    // Update knowledge graph
    update_graph(vault.knowledge_graph, note, parsed)

    RETURN Ok


FUNCTION index_fulltext(index: TantivyIndex, note: Note, parsed: ParsedMarkdown):
    writer = index.writer()

    doc = Document.new()
    doc.add_text("id", note.id)
    doc.add_text("title", parsed.frontmatter?.title OR note.basename)
    doc.add_text("content", parsed.content)

    FOR EACH tag IN parsed.tags:
        doc.add_text("tags", tag.name)

    doc.add_date("created", note.stat.created)
    doc.add_date("modified", note.stat.modified)

    writer.add_document(doc)
    writer.commit()


FUNCTION index_vectors(vector_db: RuVector, note: Note, parsed: ParsedMarkdown):
    // For long notes, split into chunks
    chunks = split_into_chunks(parsed.content, max_tokens: 512, overlap: 50)

    FOR EACH (index, chunk) IN enumerate(chunks):
        // Generate embedding
        embedding = generate_embedding(chunk)

        // Create unique ID for chunk
        chunk_id = IF chunks.length > 1:
            "{note.id}#chunk{index}"
        ELSE:
            note.id

        // Store with metadata
        vector_db.insert(
            id: chunk_id,
            vector: embedding,
            metadata: {
                note_id: note.id,
                chunk_index: index,
                title: parsed.frontmatter?.title,
                tags: parsed.tags.map(t -> t.name)
            }
        )


FUNCTION generate_embedding(text: String) -> Vector<Float>:
    // Use all-MiniLM-L6-v2 model (384 dimensions)
    // This could be local or via API
    tokens = tokenize(text, max_length: 512)
    embedding = model.encode(tokens)
    RETURN normalize(embedding)
```

---

## 5. Knowledge Graph

### 5.1 Graph Operations

```pseudocode
FUNCTION update_graph(graph: KnowledgeGraph, note: Note, parsed: ParsedMarkdown):
    // Ensure node exists
    IF note.id NOT IN graph.nodes:
        node = NoteNode {
            id: note.id,
            title: parsed.frontmatter?.title OR note.basename,
            tags: parsed.tags.map(t -> t.name)
        }
        node_index = graph.graph.add_node(node)
        graph.nodes[note.id] = node_index
    ELSE:
        // Update existing node
        node_index = graph.nodes[note.id]
        graph.graph[node_index].title = parsed.frontmatter?.title OR note.basename
        graph.graph[node_index].tags = parsed.tags.map(t -> t.name)

    // Remove old outgoing edges
    old_edges = graph.graph.edges_from(node_index)
    FOR EACH edge IN old_edges:
        graph.graph.remove_edge(edge)

    // Clear old resolved links from this note
    graph.resolved_links[note.id] = Map.new()

    // Add new edges for links
    FOR EACH link IN parsed.links:
        target_note = resolve_link_target(graph, link.target)

        IF target_note != NULL:
            // Add resolved link
            target_index = graph.nodes[target_note.id]
            graph.graph.add_edge(node_index, target_index, LinkEdge {
                link_type: REFERENCE,
                weight: 1.0
            })

            // Update resolved links count
            count = graph.resolved_links[note.id].get(target_note.id, 0)
            graph.resolved_links[note.id][target_note.id] = count + 1
        ELSE:
            // Track unresolved link
            count = graph.unresolved_links[note.id].get(link.target, 0)
            graph.unresolved_links[note.id][link.target] = count + 1

    // Add edges for embeds
    FOR EACH embed IN parsed.embeds:
        target_note = resolve_link_target(graph, embed.target)
        IF target_note != NULL:
            target_index = graph.nodes[target_note.id]
            graph.graph.add_edge(node_index, target_index, LinkEdge {
                link_type: EMBED,
                weight: 1.5  // Embeds have higher weight
            })


FUNCTION get_backlinks(graph: KnowledgeGraph, note_id: String) -> List<String>:
    IF note_id NOT IN graph.nodes:
        RETURN []

    node_index = graph.nodes[note_id]
    backlinks = []

    // Get all incoming edges
    FOR EACH source_index IN graph.graph.neighbors_directed(node_index, INCOMING):
        source_node = graph.graph[source_index]
        backlinks.push(source_node.id)

    RETURN backlinks


FUNCTION get_forward_links(graph: KnowledgeGraph, note_id: String) -> List<String>:
    IF note_id NOT IN graph.nodes:
        RETURN []

    node_index = graph.nodes[note_id]
    forward_links = []

    // Get all outgoing edges
    FOR EACH target_index IN graph.graph.neighbors_directed(node_index, OUTGOING):
        target_node = graph.graph[target_index]
        forward_links.push(target_node.id)

    RETURN forward_links


FUNCTION get_graph_visualization_data(
    graph: KnowledgeGraph,
    center: String OR NULL,
    depth: Integer,
    filters: GraphFilters
) -> GraphData:

    nodes_to_include = Set<String>.new()

    IF center != NULL:
        // Local graph - BFS from center node
        nodes_to_include = bfs_collect(graph, center, depth)
    ELSE:
        // Global graph - all nodes
        nodes_to_include = graph.nodes.keys().to_set()

    // Apply filters
    IF filters.tags.is_not_empty():
        nodes_to_include = nodes_to_include.filter(id ->
            graph.graph[graph.nodes[id]].tags.intersects(filters.tags)
        )

    IF filters.exclude_orphans:
        nodes_to_include = nodes_to_include.filter(id ->
            graph.graph.degree(graph.nodes[id]) > 0
        )

    // Build visualization data
    vis_nodes = []
    vis_edges = []

    FOR EACH id IN nodes_to_include:
        node = graph.graph[graph.nodes[id]]
        link_count = graph.graph.degree(graph.nodes[id])

        vis_nodes.push({
            id: id,
            label: node.title,
            size: calculate_node_size(link_count),
            color: calculate_node_color(node.tags)
        })

    FOR EACH edge IN graph.graph.edges():
        source_id = graph.graph[edge.source()].id
        target_id = graph.graph[edge.target()].id

        IF source_id IN nodes_to_include AND target_id IN nodes_to_include:
            vis_edges.push({
                source: source_id,
                target: target_id,
                type: edge.weight().link_type
            })

    RETURN GraphData { nodes: vis_nodes, edges: vis_edges }
```

---

## 6. File Watcher

### 6.1 File Watch Handler

```pseudocode
FUNCTION setup_file_watcher(vault: Vault) -> FileWatcher:
    watcher = notify::Watcher.new()

    // Watch vault directory recursively
    watcher.watch(vault.path, RECURSIVE)

    // Debounce events
    pending_events = Map<Path, FileEvent>.new()
    debounce_timer = NULL

    watcher.on_event((event) ->
        // Store event
        pending_events[event.path] = event

        // Reset debounce timer
        IF debounce_timer != NULL:
            cancel(debounce_timer)

        debounce_timer = schedule_after(500ms, () ->
            process_pending_events(vault, pending_events)
            pending_events.clear()
        )
    )

    RETURN watcher


FUNCTION process_pending_events(vault: Vault, events: Map<Path, FileEvent>):
    FOR EACH (path, event) IN events:
        // Skip non-markdown files (unless attachment)
        IF NOT is_markdown(path) AND NOT is_supported_attachment(path):
            CONTINUE

        // Skip .obsidian directory (config changes handled separately)
        IF path.contains(".obsidian"):
            CONTINUE

        MATCH event.kind:
            CASE CREATE:
                handle_file_created(vault, path)

            CASE MODIFY:
                handle_file_modified(vault, path)

            CASE DELETE:
                handle_file_deleted(vault, path)

            CASE RENAME(old_path, new_path):
                handle_file_renamed(vault, old_path, new_path)


FUNCTION handle_file_created(vault: Vault, path: Path):
    // Read file content
    content = read_file(path)

    // Create note object
    relative_path = path.relative_to(vault.path)
    note = Note {
        id: relative_path,
        path: path,
        basename: stem(path),
        extension: extension(path),
        content: content,
        stat: get_file_stat(path)
    }

    // Parse and extract metadata
    parsed = parse_markdown(content)
    note.frontmatter = parsed.frontmatter

    // Index the new note
    index_note(note, vault)

    // Emit event for UI
    vault.events.emit("note-created", note)


FUNCTION handle_file_modified(vault: Vault, path: Path):
    relative_path = path.relative_to(vault.path)

    // Check if note exists in cache
    cached = vault.metadata_store.get(relative_path)

    // Read new content
    new_content = read_file(path)

    // Check if content actually changed (avoid false triggers)
    new_hash = hash(new_content)
    IF cached != NULL AND cached.content_hash == new_hash:
        RETURN  // No actual change

    // Re-index
    note = Note {
        id: relative_path,
        path: path,
        content: new_content,
        stat: get_file_stat(path)
    }

    parsed = parse_markdown(new_content)
    note.frontmatter = parsed.frontmatter

    // Update all indexes
    PARALLEL:
        update_fulltext_index(vault.fulltext_index, note, parsed)
        update_vector_index(vault.vector_db, note, parsed)
        update_metadata_store(vault.metadata_store, note, parsed)
        update_graph(vault.knowledge_graph, note, parsed)

    // Emit event
    vault.events.emit("note-modified", note)


FUNCTION handle_file_deleted(vault: Vault, path: Path):
    relative_path = path.relative_to(vault.path)

    // Remove from all indexes
    vault.fulltext_index.delete(relative_path)
    vault.vector_db.delete(relative_path)
    vault.metadata_store.delete(relative_path)

    // Update graph (remove node and edges)
    IF relative_path IN vault.knowledge_graph.nodes:
        node_index = vault.knowledge_graph.nodes[relative_path]
        vault.knowledge_graph.graph.remove_node(node_index)
        vault.knowledge_graph.nodes.remove(relative_path)

    // Update unresolved links (notes that linked to deleted note)
    backlinks = get_backlinks(vault.knowledge_graph, relative_path)
    FOR EACH source_id IN backlinks:
        // Mark as unresolved
        vault.knowledge_graph.unresolved_links[source_id][relative_path] =
            vault.knowledge_graph.resolved_links[source_id].remove(relative_path) OR 1

    // Emit event
    vault.events.emit("note-deleted", { id: relative_path })
```

---

## 7. Plugin System

### 7.1 Plugin Runtime

```pseudocode
STRUCTURE PluginRuntime:
    runtime: DenoCore.JsRuntime
    plugins: Map<String, PluginInstance>
    api_bindings: ObsidianApiBindings

FUNCTION create_plugin_runtime(vault: Vault) -> PluginRuntime:
    // Create V8 runtime with extensions
    extensions = [
        create_obsidian_extension(vault)
    ]

    runtime = DenoCore.JsRuntime.new({
        extensions: extensions,
        will_snapshot: FALSE
    })

    // Initialize Obsidian API globals
    runtime.execute_script("obsidian_globals.js", """
        globalThis.obsidian = {
            Plugin: class Plugin {
                constructor() {
                    this.app = globalThis.__obsidian_app;
                }

                onload() {}
                onunload() {}

                addCommand(command) {
                    return Deno.core.ops.op_add_command(command);
                }

                addRibbonIcon(icon, title, callback) {
                    const id = Deno.core.ops.op_add_ribbon_icon(icon, title);
                    globalThis.__ribbon_callbacks[id] = callback;
                    return { id };
                }

                registerEvent(eventRef) {
                    // Auto-cleanup on unload
                    this._events = this._events || [];
                    this._events.push(eventRef);
                }

                async loadData() {
                    return Deno.core.ops.op_load_plugin_data(this.manifest.id);
                }

                async saveData(data) {
                    return Deno.core.ops.op_save_plugin_data(this.manifest.id, data);
                }
            },

            Modal: class Modal {
                constructor(app) {
                    this.app = app;
                }
                open() { Deno.core.ops.op_modal_open(this); }
                close() { Deno.core.ops.op_modal_close(); }
            },

            // ... more API classes
        };
    """)

    RETURN PluginRuntime { runtime, plugins: Map.new() }


FUNCTION load_plugin(runtime: PluginRuntime, plugin_path: Path) -> Result<()>:
    // Read manifest
    manifest_path = plugin_path / "manifest.json"
    manifest = parse_json(read_file(manifest_path))

    // Validate manifest
    IF manifest.id == NULL OR manifest.name == NULL:
        RETURN Error("Invalid plugin manifest")

    // Read main script
    main_path = plugin_path / (manifest.main OR "main.js")
    main_script = read_file(main_path)

    // Load plugin in sandboxed context
    runtime.runtime.execute_script(
        "{manifest.id}/main.js",
        """
        (function() {
            const exports = {};
            const module = { exports };

            // Plugin code
            {main_script}

            // Get default export (Plugin class)
            const PluginClass = module.exports.default || module.exports;

            // Instantiate and register
            const plugin = new PluginClass();
            plugin.manifest = {manifest_json};
            globalThis.__plugins['{manifest.id}'] = plugin;
        })();
        """
    )

    // Call onload
    runtime.runtime.execute_script(
        "{manifest.id}/init.js",
        "globalThis.__plugins['{manifest.id}'].onload();"
    )

    runtime.plugins[manifest.id] = PluginInstance {
        manifest: manifest,
        enabled: TRUE
    }

    RETURN Ok


FUNCTION unload_plugin(runtime: PluginRuntime, plugin_id: String) -> Result<()>:
    IF plugin_id NOT IN runtime.plugins:
        RETURN Error("Plugin not loaded")

    // Call onunload
    runtime.runtime.execute_script(
        "{plugin_id}/unload.js",
        """
        const plugin = globalThis.__plugins['{plugin_id}'];
        if (plugin && plugin.onunload) {
            plugin.onunload();
        }
        // Cleanup registered events
        if (plugin._events) {
            plugin._events.forEach(e => e.unsubscribe());
        }
        delete globalThis.__plugins['{plugin_id}'];
        """
    )

    runtime.plugins.remove(plugin_id)

    RETURN Ok
```

### 7.2 Obsidian API Bindings

```pseudocode
FUNCTION create_obsidian_extension(vault: Vault) -> DenoExtension:
    ops = [
        // Vault operations
        op_vault_read: async (path) ->
            content = vault.read_note(path).await
            RETURN content,

        op_vault_modify: async (path, content) ->
            vault.modify_note_by_path(path, content).await
            RETURN NULL,

        op_vault_create: async (path, content) ->
            note = vault.create_note(path, content).await
            RETURN note.id,

        op_vault_delete: async (path) ->
            vault.delete_note_by_path(path).await
            RETURN NULL,

        op_vault_get_files: () ->
            files = vault.get_all_files()
            RETURN files.map(f -> { path: f.id, name: f.basename }),

        // Workspace operations
        op_workspace_get_active_file: () ->
            active = vault.workspace.get_active_file()
            RETURN active?.id,

        op_workspace_open_file: async (path, new_leaf) ->
            vault.workspace.open_file(path, new_leaf).await
            RETURN NULL,

        // Metadata operations
        op_metadata_get_cache: (path) ->
            cache = vault.metadata_store.get_cache(path)
            RETURN cache,

        op_metadata_get_backlinks: (path) ->
            backlinks = vault.knowledge_graph.get_backlinks(path)
            RETURN backlinks,

        // Command operations
        op_add_command: (command) ->
            id = vault.commands.register(command)
            RETURN id,

        // Ribbon operations
        op_add_ribbon_icon: (icon, title) ->
            id = vault.ui.add_ribbon_icon(icon, title)
            RETURN id,

        // Plugin data operations
        op_load_plugin_data: async (plugin_id) ->
            data = vault.read_plugin_data(plugin_id).await
            RETURN parse_json(data),

        op_save_plugin_data: async (plugin_id, data) ->
            vault.write_plugin_data(plugin_id, stringify_json(data)).await
            RETURN NULL,
    ]

    RETURN DenoExtension.builder("obsidian_api")
        .ops(ops)
        .build()
```

---

## 8. Workspace Management

### 8.1 Workspace State

```pseudocode
STRUCTURE WorkspaceManager:
    root: WorkspaceSplit
    left_sidebar: Sidebar
    right_sidebar: Sidebar
    active_leaf_id: String OR NULL
    leaves: Map<String, WorkspaceLeaf>

FUNCTION open_file(workspace: WorkspaceManager, file_path: String, options: OpenOptions) -> WorkspaceLeaf:
    // Determine target leaf
    target_leaf = IF options.new_leaf:
        create_new_leaf(workspace)
    ELSE IF options.split:
        split_active_leaf(workspace, options.split_direction)
    ELSE:
        get_or_create_active_leaf(workspace)

    // Set leaf state
    target_leaf.view_type = "markdown"
    target_leaf.state = {
        file: file_path,
        mode: options.mode OR "source"
    }

    // Load file content
    target_leaf.content = workspace.vault.read_note(file_path)

    // Set as active
    workspace.active_leaf_id = target_leaf.id

    // Emit event
    workspace.events.emit("file-open", { leaf: target_leaf, file: file_path })

    RETURN target_leaf


FUNCTION split_leaf(workspace: WorkspaceManager, leaf_id: String, direction: SplitDirection) -> WorkspaceLeaf:
    leaf = workspace.leaves[leaf_id]
    parent = find_parent(workspace.root, leaf_id)

    // Create new leaf
    new_leaf = WorkspaceLeaf {
        id: generate_id(),
        view_type: leaf.view_type,
        state: deep_clone(leaf.state)
    }
    workspace.leaves[new_leaf.id] = new_leaf

    // Update parent to be a split
    IF parent.type == "leaf":
        // Convert leaf to split
        new_split = WorkspaceSplit {
            type: "split",
            direction: direction,
            children: [leaf, new_leaf]
        }
        replace_in_tree(workspace.root, leaf_id, new_split)
    ELSE:
        // Add to existing split
        IF parent.direction == direction:
            // Same direction, just add
            parent.children.push(new_leaf)
        ELSE:
            // Different direction, nest
            new_split = WorkspaceSplit {
                type: "split",
                direction: direction,
                children: [leaf, new_leaf]
            }
            index = parent.children.find_index(c -> c.id == leaf_id)
            parent.children[index] = new_split

    RETURN new_leaf


FUNCTION save_workspace(workspace: WorkspaceManager, name: String OR NULL):
    state = serialize_workspace(workspace)

    IF name == NULL:
        // Save to default workspace.json
        path = workspace.vault.path / ".obsidian" / "workspace.json"
    ELSE:
        // Save to named workspace
        workspaces = load_workspaces_config(workspace.vault)
        workspaces[name] = state
        save_workspaces_config(workspace.vault, workspaces)
        RETURN

    write_file(path, stringify_json(state))


FUNCTION load_workspace(workspace: WorkspaceManager, name: String OR NULL):
    IF name == NULL:
        path = workspace.vault.path / ".obsidian" / "workspace.json"
        IF NOT exists(path):
            RETURN  // Use default layout
        state = parse_json(read_file(path))
    ELSE:
        workspaces = load_workspaces_config(workspace.vault)
        IF name NOT IN workspaces:
            RETURN Error("Workspace not found")
        state = workspaces[name]

    // Rebuild workspace from state
    workspace.root = deserialize_split(state.main)
    workspace.left_sidebar = deserialize_sidebar(state.left)
    workspace.right_sidebar = deserialize_sidebar(state.right)
    workspace.active_leaf_id = state.active_leaf

    // Rebuild leaves map
    workspace.leaves.clear()
    collect_leaves(workspace.root, workspace.leaves)
```

---

## 9. Configuration Management

### 9.1 App Configuration

```pseudocode
STRUCTURE AppConfig:
    editor: EditorConfig
    files: FilesConfig
    appearance: AppearanceConfig
    plugins: PluginsConfig
    hotkeys: HotkeysConfig

STRUCTURE EditorConfig:
    default_view_mode: String       // "source" | "preview" | "live"
    vim_mode: Boolean
    spell_check: Boolean
    line_numbers: Boolean
    readable_line_length: Boolean
    strict_line_breaks: Boolean
    smart_indent: Boolean
    auto_pair_brackets: Boolean
    auto_pair_markdown: Boolean

STRUCTURE FilesConfig:
    attachment_folder: String
    new_file_location: String       // "root" | "current" | "folder"
    new_file_folder: String
    default_file_format: String     // "markdown" | "canvas"
    deleted_file_destination: String // "trash" | "system" | "permanent"
    always_update_links: Boolean

FUNCTION load_config(vault_path: Path) -> AppConfig:
    config_path = vault_path / ".obsidian" / "app.json"

    IF exists(config_path):
        json = parse_json(read_file(config_path))
        config = deserialize_config(json)
    ELSE:
        config = AppConfig.default()

    RETURN config


FUNCTION save_config(vault_path: Path, config: AppConfig):
    config_path = vault_path / ".obsidian" / "app.json"

    json = serialize_config(config)
    write_file(config_path, stringify_json(json, pretty: TRUE))


FUNCTION apply_config(app: App, config: AppConfig):
    // Apply editor settings
    app.editor.set_vim_mode(config.editor.vim_mode)
    app.editor.set_spell_check(config.editor.spell_check)
    app.editor.set_line_numbers(config.editor.line_numbers)

    // Apply appearance settings
    app.ui.set_theme(config.appearance.theme)
    app.ui.set_accent_color(config.appearance.accent_color)
    app.ui.set_font_family(config.appearance.font_family)
    app.ui.set_font_size(config.appearance.font_size)

    // Load CSS snippets
    FOR EACH snippet IN config.appearance.css_snippets:
        IF snippet.enabled:
            app.ui.load_css_snippet(snippet.path)

    // Apply hotkeys
    FOR EACH (command_id, keys) IN config.hotkeys:
        app.commands.set_hotkey(command_id, keys)
```

---

## 10. Tauri Commands (IPC)

### 10.1 Command Definitions

```pseudocode
// Vault commands
#[tauri_command]
FUNCTION cmd_open_vault(app: AppHandle, path: String) -> Result<VaultInfo>:
    vault = initialize_vault(Path.from(path)).await
    app.state().set_vault(vault)
    RETURN VaultInfo {
        path: path,
        name: vault.name,
        note_count: vault.note_count()
    }

#[tauri_command]
FUNCTION cmd_get_note(app: AppHandle, path: String) -> Result<NoteData>:
    vault = app.state().vault()
    content = vault.read_note(path).await
    metadata = vault.metadata_store.get_cache(path)
    RETURN NoteData { content, metadata }

#[tauri_command]
FUNCTION cmd_save_note(app: AppHandle, path: String, content: String) -> Result<()>:
    vault = app.state().vault()
    vault.modify_note_by_path(path, content).await
    RETURN Ok

#[tauri_command]
FUNCTION cmd_create_note(app: AppHandle, path: String, content: String) -> Result<String>:
    vault = app.state().vault()
    note = vault.create_note(path, content).await
    RETURN note.id

#[tauri_command]
FUNCTION cmd_delete_note(app: AppHandle, path: String) -> Result<()>:
    vault = app.state().vault()
    vault.delete_note_by_path(path).await
    RETURN Ok

// Search commands
#[tauri_command]
FUNCTION cmd_search(app: AppHandle, query: String, limit: Integer) -> Result<List<SearchResult>>:
    vault = app.state().vault()
    results = hybrid_search(vault, query, limit).await
    RETURN results

// Graph commands
#[tauri_command]
FUNCTION cmd_get_graph_data(app: AppHandle, options: GraphOptions) -> Result<GraphData>:
    vault = app.state().vault()
    data = get_graph_visualization_data(
        vault.knowledge_graph,
        options.center,
        options.depth,
        options.filters
    )
    RETURN data

#[tauri_command]
FUNCTION cmd_get_backlinks(app: AppHandle, path: String) -> Result<List<BacklinkData>>:
    vault = app.state().vault()
    backlinks = get_backlinks(vault.knowledge_graph, path)

    result = []
    FOR EACH source_id IN backlinks:
        source_cache = vault.metadata_store.get_cache(source_id)
        links = source_cache.links.filter(l -> l.target == path)
        result.push(BacklinkData {
            source: source_id,
            title: source_cache.title,
            links: links
        })

    RETURN result

// Workspace commands
#[tauri_command]
FUNCTION cmd_open_file(app: AppHandle, path: String, options: OpenOptions) -> Result<LeafId>:
    workspace = app.state().workspace()
    leaf = open_file(workspace, path, options)
    RETURN leaf.id

#[tauri_command]
FUNCTION cmd_save_workspace(app: AppHandle, name: String OR NULL) -> Result<()>:
    workspace = app.state().workspace()
    save_workspace(workspace, name)
    RETURN Ok
```

---

**Document Status:** PSEUDOCODE COMPLETE
**Next Phase:** REFINEMENT (Implementation)
**Author:** SPARC Pseudocode Agent
**Date:** 2025-12-07
