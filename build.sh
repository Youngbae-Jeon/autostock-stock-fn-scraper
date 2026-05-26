#!/usr/bin/env bash

function build() {
	DOCKERFILE=$1
	REPOSITORY=$2
	TAG=$(date +'%Y%m%d')
	COLOR='\033[0;33m'
	NC='\033[0m' # No Color
	docker build -f $DOCKERFILE -t $REPOSITORY:$TAG . &&\
	docker tag $REPOSITORY:$TAG $REPOSITORY:latest &&\
	echo -e "${COLOR}Type the command to push:${NC} docker push $REPOSITORY:latest"
}

case $1 in
	1|dp)
		build Dockerfile repo.autostock.co.kr/autostock-stock-dp-scrap
		;;
	*)
		echo "배포용 도커 이미지를 빌드합니다."
		echo "Usage: $0 {dp|NUMBER}"
		echo "NUMBER:"
		echo "    1: stock-dp-scrap"
		;;
esac
