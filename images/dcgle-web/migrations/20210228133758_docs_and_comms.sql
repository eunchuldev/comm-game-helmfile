-- Add migration script here

CREATE FUNCTION vgram(text text, min integer default 2, max integer default 20) RETURNS tsvector AS $BODY$ BEGIN
  RETURN array_to_tsvector((select array_agg(substring(lexeme, l1, l2)) from unnest(string_to_array(REGEXP_REPLACE(text, '[^A-Za-z가-힣ㅏ-ㅣㄱ-ㅎ0-9*;(),.@~#\s]', ''), ' ')) lexeme, generate_series(1, length(lexeme)) l1, generate_series(greatest(1, min), least(length(lexeme)-l1+1, max)) l2));
END; $BODY$ IMMUTABLE language plpgsql;

CREATE FUNCTION cat_and_time(cat text, t timestamp with time zone) RETURNS bigint AS $BODY$ BEGIN
  RETURN (( ('x' || substr(md5(cat), 1, 8))::bit(32)::bigint << 32 ) + cast(extract(epoch from t) as int)::bigint);
END; $BODY$ IMMUTABLE language plpgsql;


CREATE TABLE dcinside_document (
  gallery_id TEXT,
  id BIGINT,
  title TEXT NOT NULL DEFAULT '',
  subject TEXT,
  _title tsvector GENERATED ALWAYS AS ( vgram(title) ) STORED,
  author_nickname TEXT NOT NULL,
  author_id TEXT,
  author_ip TEXT,
  comment_count INT NOT NULL DEFAULT 0,
  like_count INT NOT NULL DEFAULT 0,
  view_count INT NOT NULL DEFAULT 0,
  kind TEXT,
  is_recommend BOOLEAN, 
  created_at TIMESTAMPTZ NOT NULL,
  _gallery_id_and_created_at BIGINT GENERATED ALWAYS AS ( cat_and_time(gallery_id, created_at) ) STORED,
  PRIMARY KEY (gallery_id, id)
);

CREATE INDEX ON dcinside_document USING rum (_title rum_tsvector_addon_ops, created_at) WITH (attach = 'created_at', to = '_title', order_by_attach = 't');
CREATE INDEX ON dcinside_document USING rum (_title rum_tsvector_addon_ops, _gallery_id_and_created_at) WITH (attach = '_gallery_id_and_created_at', to = '_title', order_by_attach = 't');

CREATE INDEX ON dcinside_document (author_nickname, created_at);
CREATE INDEX ON dcinside_document (author_id, created_at);
CREATE INDEX ON dcinside_document (author_ip, created_at);

CREATE INDEX ON dcinside_document (author_nickname, gallery_id, created_at);
CREATE INDEX ON dcinside_document (author_id, gallery_id, created_at);
CREATE INDEX ON dcinside_document (author_ip, gallery_id, created_at);

CREATE TABLE dcinside_comment (
  gallery_id TEXT,
  id BIGINT,
  document_id BIGINT,
  contents TEXT NOT NULL DEFAULT '',
  _contents tsvector GENERATED ALWAYS AS ( vgram(contents) ) STORED,
  author_nickname TEXT NOT NULL,
  author_id TEXT,
  author_ip TEXT,
  created_at TIMESTAMPTZ NOT NULL,
  _gallery_id_and_created_at BIGINT GENERATED ALWAYS AS ( cat_and_time(gallery_id, created_at) ) STORED,
  PRIMARY KEY (gallery_id, id)
);

CREATE INDEX ON dcinside_comment USING rum (_contents rum_tsvector_addon_ops, created_at) WITH (attach = 'created_at', to = '_contents', order_by_attach = 't');
CREATE INDEX ON dcinside_comment USING rum (_contents rum_tsvector_addon_ops, _gallery_id_and_created_at) WITH (attach = '_gallery_id_and_created_at', to = '_contents', order_by_attach = 't');

CREATE INDEX ON dcinside_comment (author_nickname, created_at);
CREATE INDEX ON dcinside_comment (author_id, created_at);
CREATE INDEX ON dcinside_comment (author_ip, created_at);

CREATE INDEX ON dcinside_comment (author_nickname, gallery_id, created_at);
CREATE INDEX ON dcinside_comment (author_id, gallery_id, created_at);
CREATE INDEX ON dcinside_comment (author_ip, gallery_id, created_at);

