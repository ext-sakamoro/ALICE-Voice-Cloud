-- Voice Cloud domain tables
create table if not exists public.audio_jobs (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    job_type text not null check (job_type in ('compress', 'decompress', 'analyze', 'tts', 'stt')),
    status text not null default 'pending' check (status in ('pending', 'processing', 'completed', 'failed')),
    input_format text,
    output_format text,
    input_size_bytes bigint,
    output_size_bytes bigint,
    compression_ratio double precision,
    duration_ms bigint,
    sample_rate integer,
    channels smallint,
    metadata jsonb default '{}',
    created_at timestamptz default now(),
    completed_at timestamptz
);
create table if not exists public.voice_models (
    id uuid primary key default gen_random_uuid(),
    name text not null,
    language text not null,
    gender text check (gender in ('male', 'female', 'neutral')),
    style text default 'neutral',
    sample_rate integer default 24000,
    is_public boolean default true,
    created_at timestamptz default now()
);
create index idx_audio_jobs_user on public.audio_jobs(user_id);
create index idx_audio_jobs_status on public.audio_jobs(status);
