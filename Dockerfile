FROM rustlang/rust:nightly-buster

WORKDIR /src

ADD https://github.com/nathanielc/pMARS/archive/master.zip /src/
RUN unzip master.zip && ls -la &&  cd /src/pMARS-master/src && make

ADD migrations /src/migrations
ADD src /src/src
ADD templates /src/templates
ADD Cargo.toml /src/
ADD Cargo.lock /src/
#RUN cargo generate-lockfile
ADD diesel.toml /src/
ADD Rocket.toml /src/
RUN cargo build --release

FROM ubuntu
RUN apt-get -y update
RUN apt-get -y upgrade
RUN apt-get install -y sqlite3 libsqlite3-dev
WORKDIR /srv/
COPY --from=0 /src/target/release/hillfort /bin/hillfort
COPY --from=0 /src/templates /srv/templates
COPY --from=0 /src/pMARS-master/src/pmars /bin/pmars-server
COPY --from=0 /src/Rocket.toml /srv/Rocket.toml
#COPY --from=0 /src/Cargo.lock /srv/Cargo.lock
ENV ROCKET_ADDRESS=0.0.0.0
ENV DATABASE_URL=db/hillfort.db
VOLUME /srv/db/
USER nobody
EXPOSE 8000
ENTRYPOINT /bin/hillfort
