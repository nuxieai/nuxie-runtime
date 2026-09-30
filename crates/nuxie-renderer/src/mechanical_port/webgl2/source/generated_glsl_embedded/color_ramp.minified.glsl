#ifdef CB
g1(h0)
#ifdef Z9
I(0,uint,UD);I(1,uint,VD);I(2,uint,WD);I(3,uint,XD);
#else
I(0,R,JC);
#endif
h1
#endif
r2 I0 W(0,i,W6);i2
#ifdef CB
Y3 Z3 G4 H4 i Ff(uint k){return sc((R(k,k,k,k)>>R(16,8,0,24))&0xffu)/255.;}A1(MF,h0,F,A,q){
#ifdef Z9
J(q,F,UD,uint);J(q,F,VD,uint);J(q,F,WD,uint);J(q,F,XD,uint);R JC=R(UD,VD,WD,XD);
#else
J(q,F,JC,R);
#endif
V(W6,i);int n8=A>>1;float x=float(n8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float aa=(A&1)==0?.0:1.;if(j.tc<.0){aa=1.-aa;}uint X6=JC.y;float y=float(X6&~Gf)+aa;if((X6&uc)!=0u&&n8==0){if((X6&ba)!=0u) x=.0;else x-=vc;}if((X6&wc)!=0u&&n8==3){if((X6&ba)!=0u) x=1.;else x+=vc;}W6=Ff(n8<=1?JC.z:JC.w);f X=o8(c(x,y),2.,j.tc);
#ifdef SC
X.y=-X.y;
#endif
c0(W6);B1(X);}
#endif
#ifdef EB
H3 I3 f3(i,NF){r(W6,i);N2(W6);}
#endif
