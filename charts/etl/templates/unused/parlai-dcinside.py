input_path = "{{`{{inputs.parameters.inputPath}}`}}"
format = "{{`{{inputs.parameters.format}}`}}"
output_path = "{{`{{inputs.parameters.outputPath}}`}}"
from pyspark.sql import SparkSession, types as T, functions as F, Row
spark = (SparkSession.builder.getOrCreate())
df = spark.read.format(format).load(input_path)
def dccon_parse(df, col):
    return df.withColumn(col, F.when(
        F.col(col).startswith('<video'),
        F.concat(
            F.lit('<dccon> '),
            F.regexp_extract(col, r'data-src="[^?]*\?no=([^"]+)"', 1),
            F.lit(' '),
            F.regexp_extract(col, r'title="([^"]*)"', 1))
    ).when(
        F.col(col).startswith('<img'),
        F.concat(
            F.lit('<dccon> '),
            F.regexp_extract(col, r'src="[^?]*\?no=([^"]+)"', 1),
            F.lit(' '),
            F.regexp_extract(col, r'title="([^"]*)"', 1))
    ).otherwise(F.col(col)))

d2c_df = df.selectExpr('gallery_id', 'title', 'author', 'EXPLODE(comments) as comment')
d2c_df = d2c_df.selectExpr('gallery_id', 'title', 'author', 'comment.contents as comment', 'comment.author as comm_author')\
    .filter(F.col('author') != F.col('comm_author'))\
    .select('gallery_id', 'title', 'comment')
d2c_df = d2c_df.filter((~F.col('comment').startswith('<div')) & (~F.col('comment').isNull()))
d2c_df = d2c_df.withColumn('title', F.regexp_replace('title', r'[\s\n\t]+', ' '))
d2c_df = d2c_df.withColumn('comment', F.regexp_replace('comment', r'[\s\n\t]+', ' '))
d2c_df = dccon_parse(d2c_df, 'comment')
d2c_df = d2c_df.selectExpr('''CONCAT("text:", gallery_id, "¶", title, '\t',
                              'labels:', comment, '\t', "episode:done") AS episode ''')
d2c_df = d2c_df.distinct()


c2c_df = df.selectExpr(
    'gallery_id', 'id as document_id', 'author as document_author',
    'EXPLODE(comments) AS comment')\
    .select('gallery_id', 'document_id', 'document_author', 'comment.author', 'comment.contents',
            'comment.created_at', F.coalesce('comment.parent_id', 'comment.id').alias('root_id'))
c2c_df = c2c_df.filter((~F.col('contents').startswith('<div')) & (~F.col('contents').isNull()))
window = Window.partitionBy('gallery_id', 'document_id', 'root_id').orderBy('created_at')
c2c_df = c2c_df.withColumn('dialog_size', F.count('*').over(Window.partitionBy('gallery_id', 'document_id', 'root_id'))).filter(F.col('dialog_size') > 1)
c2c_df = c2c_df.withColumn('contents', F.regexp_replace('contents', r'[\s\n\t]+', ' '))
c2c_df = dccon_parse(c2c_df, 'contents')
c2c_df = c2c_df.withColumn('is_root_author', F.first(F.col('author')).over(window) == F.col('author'))
c2c_df = c2c_df.withColumn('is_document_author', F.col('author') == F.col('document_author'))
c2c_df = c2c_df.withColumn(
    'dialog_id',
    F.sum((F.col('is_root_author') != F.lag('is_root_author', 1).over(window)).cast('int')).over(window))
c2c_df = c2c_df\
    .groupBy('gallery_id', 'document_id', 'root_id', 'dialog_id', 'is_root_author').agg(
        F.concat_ws(". ", F.collect_list('contents')).alias('contents'),
        F.first('created_at').alias('created_at'))\
    .withColumn('contents', F.concat(
        F.when(F.col('is_root_author'), F.lit('text:')).otherwise(F.lit('labels:')),
        'contents'))
c2c_df = c2c_df\
    .groupBy('gallery_id', 'document_id', 'root_id').agg(
        F.max('dialog_id').alias('dialog_real_size'),
        F.concat(F.concat_ws("\t", F.collect_list('contents')).alias('contents'), F.lit("\tepisode:done")).alias('contents'),
    )\
    .filter(F.col('dialog_real_size') > 1)\
    .select('contents')
c2c_df = c2c_df.distinct()

d2c_df.union(c2c_df).coalesce(1).write\
  .mode('overwrite')\
  .format('text')\
  .option("compression","gzip")\
  .save(output_path)
