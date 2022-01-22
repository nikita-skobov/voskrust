FROM ubuntu:20.04

# updates, and fetch dependency code
RUN apt update && apt-get install -y software-properties-common wget
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-get update
RUN apt-add-repository "deb http://apt.llvm.org/bionic/ llvm-toolchain-bionic-6.0 main"
RUN apt-get install -y clang-6.0 lld-6.0
RUN apt-get install -y automake autoconf libtool cmake git libatomic-ops-dev
RUN cd opt/ && git clone -b vosk --single-branch https://github.com/alphacep/kaldi && cd kaldi/tools/ && \
    git clone -b v0.3.13 --single-branch https://github.com/xianyi/OpenBLAS && \
    git clone -b v3.2.1 --single-branch https://github.com/alphacep/clapack

# building openblas and clapack
RUN cd /opt/kaldi/tools && \
    CXX=clang++-6.0 CC=clang-6.0 make -C OpenBLAS ONLY_CBLAS=1 DYNAMIC_ARCH=1 TARGET=NEHALEM USE_LOCKING=1 USE_THREAD=0 all && \
    CXX=clang++-6.0 CC=clang-6.0 make -C OpenBLAS PREFIX=$(pwd)/OpenBLAS/install install && \
    mkdir -p clapack/BUILD && \
    cd clapack/BUILD/ && \
    CXX=clang++-6.0 CC=clang-6.0 cmake ..
RUN cd /opt/kaldi/tools/clapack/BUILD && \
    CXX=clang++-6.0 CC=clang-6.0 make -j 10 && \
    find . -name "*.a" | xargs cp -t ../../OpenBLAS/install/lib

# building openfst
RUN cd /opt/kaldi/tools/ && \
    git clone --single-branch https://github.com/alphacep/openfst openfst && \
    cd openfst/ && \
    autoreconf -i && \
    CXX=clang++-6.0 CC=clang-6.0 CFLAGS="-g -O3" ./configure --prefix=/opt/kaldi/tools/openfst --enable-static --enable-shared --enable-far --enable-ngram-fsts --enable-lookahead-fsts --with-pic --disable-bin && \
    CXX=clang++-6.0 CC=clang-6.0 make -j 10 && \
    CXX=clang++-6.0 CC=clang-6.0 make install

# building kaldi
RUN cd /opt/kaldi/src/ && \
    CXX=clang++-6.0 CC=clang-6.0 ./configure --mathlib=OPENBLAS_CLAPACK --shared --use-cuda=no && \
    sed -i 's:-msse -msse2:-msse -msse2:g' kaldi.mk && \
    sed -i 's: -O1 : -O3 :g' kaldi.mk && \
    CXX=clang++-6.0 CC=clang-6.0 make -j $(nproc) online2 lm rnnlm

# building vosk api
RUN cd /opt && \
    git clone https://github.com/alphacep/vosk-api && \
    cd vosk-api/src/ && \
    clang++-6.0 -g -O3 -std=c++17 -Wno-deprecated-declarations -fPIC -DFST_NO_DYNAMIC_LINKING -I. -I/opt/kaldi/src -I/opt/kaldi/tools/openfst/include   -I/opt/kaldi/tools/OpenBLAS/install/include -c -o recognizer.o recognizer.cc && \
    clang++-6.0 -g -O3 -std=c++17 -Wno-deprecated-declarations -fPIC -DFST_NO_DYNAMIC_LINKING -I. -I/opt/kaldi/src -I/opt/kaldi/tools/openfst/include   -I/opt/kaldi/tools/OpenBLAS/install/include -c -o language_model.o language_model.cc && \
    clang++-6.0 -g -O3 -std=c++17 -Wno-deprecated-declarations -fPIC -DFST_NO_DYNAMIC_LINKING -I. -I/opt/kaldi/src -I/opt/kaldi/tools/openfst/include   -I/opt/kaldi/tools/OpenBLAS/install/include -c -o model.o model.cc && \
    clang++-6.0 -g -O3 -std=c++17 -Wno-deprecated-declarations -fPIC -DFST_NO_DYNAMIC_LINKING -I. -I/opt/kaldi/src -I/opt/kaldi/tools/openfst/include   -I/opt/kaldi/tools/OpenBLAS/install/include -c -o spk_model.o spk_model.cc && \
    clang++-6.0 -g -O3 -std=c++17 -Wno-deprecated-declarations -fPIC -DFST_NO_DYNAMIC_LINKING -I. -I/opt/kaldi/src -I/opt/kaldi/tools/openfst/include   -I/opt/kaldi/tools/OpenBLAS/install/include -c -o vosk_api.o vosk_api.cc

# combining vosk api obj files into libvoskapi.a
RUN cd /opt/vosk-api/src/ && \
    ar crus libvoskapi.a recognizer.o language_model.o model.o model.o spk_model.o vosk_api.o && \
    mkdir copydir && \
    cp libvoskapi.a ./copydir

# copying all of the necessary .a libs from the kaldi built into one directory
RUN cp \
    /opt/kaldi/src/online2/kaldi-online2.a \
    /opt/kaldi/src/decoder/kaldi-decoder.a \
    /opt/kaldi/src/ivector/kaldi-ivector.a \
    /opt/kaldi/src/gmm/kaldi-gmm.a \
    /opt/kaldi/src/tree/kaldi-tree.a \
    /opt/kaldi/src/feat/kaldi-feat.a \
    /opt/kaldi/src/lat/kaldi-lat.a \
    /opt/kaldi/src/lm/kaldi-lm.a \
    /opt/kaldi/src/rnnlm/kaldi-rnnlm.a \
    /opt/kaldi/src/hmm/kaldi-hmm.a \
    /opt/kaldi/src/nnet3/kaldi-nnet3.a \
    /opt/kaldi/src/transform/kaldi-transform.a \
    /opt/kaldi/src/cudamatrix/kaldi-cudamatrix.a \
    /opt/kaldi/src/matrix/kaldi-matrix.a \
    /opt/kaldi/src/fstext/kaldi-fstext.a \
    /opt/kaldi/src/util/kaldi-util.a \
    /opt/kaldi/src/base/kaldi-base.a \
    /opt/kaldi/tools/openfst/lib/libfst.a \
    /opt/kaldi/tools/openfst/lib/libfstngram.a \
    /opt/kaldi/tools/OpenBLAS/install/lib/libopenblas.a \
    /opt/kaldi/tools/OpenBLAS/install/lib/liblapack.a \
    /opt/kaldi/tools/OpenBLAS/install/lib/libblas.a \
    /opt/kaldi/tools/OpenBLAS/install/lib/libf2c.a \
    /opt/vosk-api/src/copydir

# creating a .mri file to use for ar
RUN cd /opt/vosk-api/src/copydir && \
    echo "create libvosk.a" >> libvosk.mri && \
    echo "addlib libvoskapi.a" >> libvosk.mri && \
    echo "addlib kaldi-online2.a" >> libvosk.mri && \
    echo "addlib kaldi-decoder.a" >> libvosk.mri && \
    echo "addlib kaldi-ivector.a" >> libvosk.mri && \
    echo "addlib kaldi-gmm.a" >> libvosk.mri && \
    echo "addlib kaldi-tree.a" >> libvosk.mri && \
    echo "addlib kaldi-feat.a" >> libvosk.mri && \
    echo "addlib kaldi-lat.a" >> libvosk.mri && \
    echo "addlib kaldi-lm.a" >> libvosk.mri && \
    echo "addlib kaldi-rnnlm.a" >> libvosk.mri && \
    echo "addlib kaldi-hmm.a" >> libvosk.mri && \
    echo "addlib kaldi-nnet3.a" >> libvosk.mri && \
    echo "addlib kaldi-transform.a" >> libvosk.mri && \
    echo "addlib kaldi-cudamatrix.a" >> libvosk.mri && \
    echo "addlib kaldi-matrix.a" >> libvosk.mri && \
    echo "addlib kaldi-fstext.a" >> libvosk.mri && \
    echo "addlib kaldi-util.a" >> libvosk.mri && \
    echo "addlib kaldi-base.a" >> libvosk.mri && \
    echo "addlib libfst.a" >> libvosk.mri && \
    echo "addlib libfstngram.a" >> libvosk.mri && \
    echo "addlib libopenblas.a" >> libvosk.mri && \
    echo "addlib liblapack.a" >> libvosk.mri && \
    echo "addlib libblas.a" >> libvosk.mri && \
    echo "addlib libf2c.a" >> libvosk.mri && \
    echo "save" >> libvosk.mri && \
    echo "end" >> libvosk.mri

# combining all of the necessary .a libraries into one big static library
RUN cd /opt/vosk-api/src/copydir && ar -M <libvosk.mri

# the above creates a libvosk.a file
# which can be copied out of the docker image
# (if you make a temporary container first)
# the commands to do so would be: (on the host machine)

# id=$(docker create <name-of-image>)
# docker cp $id:/opt/vosk-api/src/copydir/libvosk.a ./
# docker rm -v $id
