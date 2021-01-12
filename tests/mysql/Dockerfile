FROM zanox/mysql

COPY schema.sql /
RUN start-mysql && \
    mysql < /schema.sql && \
    stop-mysql
