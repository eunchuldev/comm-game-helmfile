const { ApolloServer, gql, MockList } = require('apollo-server');

const { makeExecutableSchema } = require('@graphql-tools/schema');
const { addMocksToSchema, mockServer } = require('@graphql-tools/mock');
const { graphql } = require('graphql');
const casual = require('casual');
const fs = require("fs");

const schemaString = fs.readFileSync('./schema.graphql', 'utf8');

const schema = makeExecutableSchema({ typeDefs: schemaString });

const mocks = {
  Query: () => ({
    documents: () => new MockList([5, 20], () => {
      let isStaticUser = Math.random() > 0.5;
      let hasComment = Math.random() > 0.8;
      let hasLike = Math.random() > 0.8;
      return {
        galleryId: casual.word,
        id: casual.integer(1, 1000000000),
        title: casual.title,
        subject: casual.word,
        authorNickname: casual.word,
        authorIp: isStaticUser? null: casual.ip,
        authorId: isStaticUser? casual.username: null,
        commentCount: hasComment? casual.integer(1, 3000): 0,
        likeCount: hasLike? casual.integer(1, 3000): 0,
        viewCount: casual.integer(1, 30000),
        kind: casual.word,
        isRecommend: casual.boolean,
        createdAt: new Date(casual.unix_time),
      }
    }),
  })
}

const preserveResolvers = false;

const resolvers = {
  Query: {
    resolved: () => "Resolved",
  }
}

const server = new ApolloServer({
  schema, 
  resolvers,
  mocks,
})

server.listen().then(({url}) => {
  console.log(` server ready at ${url} `);
});
