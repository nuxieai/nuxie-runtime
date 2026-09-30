#ifdef VERTEX
h1(h0)
#ifdef aa
I(0,uint,UD);I(1,uint,VD);I(2,uint,WD);I(3,uint,XD);
#else
I(0,R,JC);
#endif
i1
#endif
q2 I0 W(0,i,V6);i2
#ifdef VERTEX
Y3 Z3 F4 G4 i Ff(uint k){return tc((R(k,k,k,k)>>R(16,8,0,24))&0xffu)/255.;}B1(MF,h0,F,A,q){
#ifdef aa
J(q,F,UD,uint);J(q,F,VD,uint);J(q,F,WD,uint);J(q,F,XD,uint);R JC=R(UD,VD,WD,XD);
#else
J(q,F,JC,R);
#endif
V(V6,i);int n8=A>>1;float x=float(n8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float ba=(A&1)==0?.0:1.;if(j.uc<.0){ba=1.-ba;}uint W6=JC.y;float y=float(W6&~Gf)+ba;if((W6&vc)!=0u&&n8==0){if((W6&ca)!=0u) x=.0;else x-=wc;}if((W6&xc)!=0u&&n8==3){if((W6&ca)!=0u) x=1.;else x+=wc;}V6=Ff(n8<=1?JC.z:JC.w);f X=o8(c(x,y),2.,j.uc);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
c0(V6);C1(X);}
#endif
#ifdef FRAGMENT
I3 J3 f3(i,NF){r(V6,i);M2(V6);}
#endif
