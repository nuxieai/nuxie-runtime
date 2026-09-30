#ifdef VERTEX
c1(d0)
#ifdef ta
K(0,uint,UD);K(1,uint,VD);K(2,uint,WD);K(3,uint,XD);
#else
K(0,N,KC);
#endif
d1
#endif
l2 E0 W(0,i,g7);d2
#ifdef VERTEX
j4 k4 P4 Q4 i Wf(uint l){return Pc((N(l,l,l,l)>>N(16,8,0,24))&0xffu)/255.;}r1(NF,d0,D,G,r){
#ifdef ta
L(r,D,UD,uint);L(r,D,VD,uint);L(r,D,WD,uint);L(r,D,XD,uint);N KC=N(UD,VD,WD,XD);
#else
L(r,D,KC,N);
#endif
T(g7,i);int D8=G>>1;float x=float(D8<=1?KC.x&0xffffu:KC.x>>16)/65536.;float ua=(G&1)==0?.0:1.;if(j.Qc<.0){ua=1.-ua;}uint h7=KC.y;float y=float(h7&~Xf)+ua;if((h7&Rc)!=0u&&D8==0){if((h7&va)!=0u) x=.0;else x-=Sc;}if((h7&Tc)!=0u&&D8==3){if((h7&va)!=0u) x=1.;else x+=Sc;}g7=Wf(D8<=1?KC.z:KC.w);e I=E8(c(x,y),2.,j.Qc);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
Z(g7);v1(I);}
#endif
#ifdef FRAGMENT
O3 P3 j3(i,OF){q(g7,i);Q2(g7);}
#endif
