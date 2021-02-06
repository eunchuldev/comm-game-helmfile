#!/usr/bin/julia

if size(ARGS, 1) < 2
  println("Convert parlai dataset to tokenizer dataset")
  println("Usage: main.jl :inputpath :outputpath")
  exit(0)
end
inpath = ARGS[1]
outpath = ARGS[2]

function parsechunk(s, set::Set{AbstractString})
  if startswith(s, "text:")
    s = SubString(s, 6)
  elseif startswith(s, "labels:")
    s = SubString(s, 8)
  else
    s = ""
  end
  if s in set
    return ""
  else
    push!(set, s)
    return s
  end
end

#parsechunk(s::SubString{String}) = parsechunk(String(s))

function parseln(s, set::Set{AbstractString})
  return join(filter((x) -> !isempty(x), map((i) -> parsechunk(i, set), split(s, '\t', keepempty=false))), '\n')
end

set = Set{AbstractString}()

open(outpath, "w") do outfile
  open(inpath) do infile
    for ln in eachline(infile)
      parsed = parseln(ln, set)
      if !isempty(parsed)
        println(outfile, parsed)
      end
    end
  end
end
