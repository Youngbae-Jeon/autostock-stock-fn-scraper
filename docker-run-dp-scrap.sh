#/bin/bash

. ./.env
docker run --rm -it\
 -eRUST_LOG=$RUST_LOG\
 repo.autostock.co.kr/autostock-stock-dp-scrap
