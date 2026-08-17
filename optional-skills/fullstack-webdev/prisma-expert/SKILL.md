---
name: prisma-expert
description: "Prisma ORM expert patterns — schema design, migrations, relations, transactions, raw queries, performance optimization, TypeScript."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [prisma, orm, database, postgresql, mysql, migrations, typescript]
    related_skills: [postgresql, nodejs-backend-patterns, api-design]
---

# Prisma Expert

## Schema Design

```prisma
// prisma/schema.prisma
generator client {
  provider        = "prisma-client-js"
  previewFeatures = ["fullTextSearch", "relationJoins"]
}

datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

model User {
  id        String   @id @default(cuid())
  email     String   @unique
  name      String?
  password  String   // Hashed
  role      Role     @default(USER)
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt

  posts      Post[]
  comments   Comment[]
  likes      Like[]

  @@index([email])
  @@map("users")
}

model Post {
  id        String   @id @default(cuid())
  title     String
  slug      String   @unique
  content   String?  @db.Text
  published Boolean  @default(false)
  viewCount Int      @default(0)
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt

  author    User      @relation(fields: [authorId], references: [id])
  authorId  String

  comments  Comment[]
  tags      Tag[]
  likes     Like[]

  @@index([authorId])
  @@index([published, createdAt])
  @@fulltext([title, content])
  @@map("posts")
}

model Comment {
  id        String   @id @default(cuid())
  body      String
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt

  author   User @relation(fields: [authorId], references: [id])
  authorId String
  post     Post @relation(fields: [postId], references: [id], onDelete: Cascade)
  postId   String

  @@index([postId])
  @@map("comments")
}

model Tag {
  id    String @id @default(cuid())
  name  String @unique
  posts Post[]
  @@map("tags")
}

model _PostToTag {
  Post   Post   @relation(fields: [postId], references: [id])
  postId String
  Tag    Tag    @relation(fields: [tagId], references: [id])
  tagId  String
  @@id([postId, tagId])
}

enum Role {
  ADMIN
  EDITOR
  USER
}
```

---

## Migrations

```bash
# Create migration from schema changes
npx prisma migrate dev --name add_posts_table

# Apply migrations in production
npx prisma migrate deploy

# Reset (dev only — NEVER in production)
npx prisma migrate reset

# Generate client after schema changes
npx prisma generate

# Studio (GUI for data)
npx prisma studio
```

---

## Query Patterns

```typescript
// Basic CRUD
const user = await prisma.user.findUnique({ where: { id: userId } });
const users = await prisma.user.findMany({
  where: { role: 'USER', createdAt: { gte: thirtyDaysAgo } },
  orderBy: { createdAt: 'desc' },
  take: 20,
  skip: offset,
});

// Include relations
const posts = await prisma.post.findMany({
  where: { published: true },
  include: {
    author: { select: { id: true, name: true, email: true } },
    tags: true,
    _count: { select: { comments: true, likes: true } },
  },
});

// Create with relations
const post = await prisma.post.create({
  data: {
    title: 'My Post',
    slug: 'my-post',
    authorId: userId,
    tags: { connect: [{ name: 'typescript' }] },
  },
});
```

---

## Complex Queries

```typescript
// Paginated cursor-based
async function getPosts(cursor?: string, limit = 20) {
  return prisma.post.findMany({
    take: limit + 1, // Fetch one extra to determine hasNext
    where: {
      published: true,
      ...(cursor && { createdAt: { lt: new Date(cursor) } }),
    },
    orderBy: { createdAt: 'desc' },
    include: {
      author: { select: { id: true, name: true } },
      _count: { select: { comments: true } },
    },
  }).then(posts => ({
    data: posts.slice(0, limit),
    nextCursor: posts.length > limit
      ? posts[limit - 1].createdAt.toISOString()
      : undefined,
  }));
}

// Aggregation
const stats = await prisma.post.aggregate({
  where: { authorId },
  _count: { _all: true },
  _sum: { viewCount: true },
  _avg: { viewCount: true },
  _min: { createdAt: true },
  _max: { createdAt: true },
});

// Group by
const byMonth = await prisma.post.groupBy({
  by: ['published'],
  _count: { _all: true },
  where: {
    createdAt: { gte: startOfYear },
  },
});
```

---

## Transactions

```typescript
// Sequential operations (implicit transaction)
const [post, comment] = await prisma.$transaction([
  prisma.post.create({ data: { title, slug, authorId } }),
  prisma.comment.create({
    data: { body: 'First!', postId: (await prisma.post.findFirst({ orderBy: { createdAt: 'desc' } }))!.id, authorId: userId }
  }),
]);

// With interactive transactions
const result = await prisma.$transaction(async (tx) => {
  const user = await tx.user.update({
    where: { id: userId },
    data: { posts: { increment: 1 } },
  });

  if (user.posts > MAX_FREE_POSTS) {
    throw new Error('Post limit reached');
  }

  return tx.post.create({
    data: { title, slug, authorId: userId },
  });
});

// Long-running with timeouts
await prisma.$transaction(async (tx) => {
  // Complex operations
}, {
  timeout: 30_000, // 30 seconds
  isolationLevel: 'Serializable',
});
```

---

## Raw Queries

```typescript
// When Prisma can't express the query
const result = await prisma.$queryRaw`
  SELECT u.id, u.name, COUNT(p.id) as post_count
  FROM users u
  LEFT JOIN posts p ON p.author_id = u.id
  WHERE u.role = 'EDITOR'
  GROUP BY u.id
  HAVING COUNT(p.id) > 5
  ORDER BY post_count DESC
  LIMIT 10
`;

// Raw for writes
await prisma.$executeRaw`UPDATE posts SET view_count = view_count + 1 WHERE id = ${postId}`;

// Full-text search (PostgreSQL)
const searchResults = await prisma.post.findMany({
  where: {
    OR: [
      { title: { search: query } },
      { content: { search: query } },
    ],
  },
});
```

---

## Performance Tips

```typescript
// Select only needed fields
const posts = await prisma.post.findMany({
  select: { id: true, title: true, slug: true, createdAt: true },
  where: { published: true },
});

// Use findFirst instead of findUnique when order matters
const latestPost = await prisma.post.findFirst({
  where: { authorId, published: true },
  orderBy: { createdAt: 'desc' },
});

// Chunk large deletes
async function deleteOldPosts(daysOld: number) {
  const threshold = new Date(Date.now() - daysOld * 24 * 60 * 60 * 1000);
  let deleted = 0;
  let total;

  do {
    const result = await prisma.post.deleteMany({
      where: { createdAt: { lt: threshold }, id: { notIn: [] } },
      take: 1000,
    });
    deleted += result.count;
  } while (deleted > 0);

  return deleted;
}

// Connection pool (in URL)
DATABASE_URL="postgresql://user:pass@host:5432/db?connection_limit=10&pool_timeout=10"
```

---

## TypeScript Integration

```typescript
// Type from Prisma
type PostWithAuthor = Awaited<ReturnType<typeof getPost>>;

// Custom result type
const result = await prisma.post.findMany({
  where: { published: true },
  select: {
    id: true,
    title: true,
    slug: true,
    author: { select: { name: true } },
  },
});
type PostSummary = typeof result[number];
```

---

## Best Practices

1. **Always** use `$transaction` for multi-step writes
2. **Always** use `select` or `include` explicitly — avoid fetching everything
3. **Always** add indexes for fields in `where` clauses
4. **Use** `findFirst` with ordering instead of `findUnique` when order matters
5. **Use** raw queries only when Prisma can't express the query
6. **Never** use `prisma.user.delete({ where: { id } })` — soft delete instead
7. **Always** run `prisma generate` after schema changes in CI
8. **Use** `prisma migrate deploy` in production, never `migrate dev`
