#define Th 10
#ifdef DB
f1(f0)J(0,f,MD);J(1,f,ND);J(2,f,VC);
#ifdef aa
J(3,uint,IE);J(4,uint,JE);J(5,uint,KE);J(6,uint,LE);
#else
J(3,X,TB);
#endif
g1
#endif
p2 H0 V(0,f,A6);H0 V(1,f,B6);H0 V(2,f,O4);H0 V(3,Q,P4);S2 V(4,uint,I7);h2
#ifdef DB
V3 i6(d3,i7,YC);W3 d4(i7,fa)B4 J4(Yc,Dg,PB);J4(Zc,Eg,ID);C4 y1(GG,f0,F,B,v){K(v,F,MD,f);K(v,F,ND,f);K(v,F,VC,f);
#ifdef aa
K(v,F,IE,uint);K(v,F,JE,uint);K(v,F,KE,uint);K(v,F,LE,uint);X TB=X(IE,JE,KE,LE);
#else
K(v,F,TB,X);
#endif
T(A6,f);T(B6,f);T(O4,f);T(P4,Q);T(I7,uint);c w0=MD.xy;c A0=MD.zw;c E0=ND.xy;c L0=ND.zw;bool ge=B<4;float y=ge?VC.z:VC.w;int kb=int(ge?TB.x:TB.y);
#ifdef Ac
int he=kb<<16;if(TB.z==0xffffffffu){--he;}float c9=float(he>>16);
#else
float c9=float(kb<<16>>16);
#endif
float d9=float(kb>>16);c q2=c((B&1)==0?c9:d9,(B&2)==0?y+1.:y);if((d9-c9)*l.Cd<.0){q2.y=2.*y+1.-q2.y;}uint R2=TB.z&0x3ffu;uint ie=(TB.z>>10)&0x3ffu;uint m2=TB.z>>20;uint i0=TB.w;uint G8=i0&Uc;uint m0=G8>0u?K0(ID,max(G8,1u)-1u).z:0u;X L4=m0!=0u?K0(PB,m0*4u+1u):X(0u,0u,0u,0u);float L2=uintBitsToFloat(L4.z);float M2=uintBitsToFloat(L4.w);if(M2!=.0&&L2==.0){float je;float Uh=Bf(w0,A0,E0,L0,je);float lb=M2*(1./ta);float Vh=wf(w0,A0,E0,L0,je,lb);float J7=1.-Vh*(1./E3);float Wh=dot(L0-w0,L0-w0)/(lb*lb);float Xh=(Wh-1.)*.5;J7=min(J7,Xh);J7=min(J7,.99);float Yh=.5*J7;float x=zc(Yh)*-2.+1.;float ke=l8(x*M2,Uh);f le=mix(w0.xyxy,L0.xyxy,f(1./3.,1./3.,2./3.,2./3.));A0=mix(A0,le.xy,ke);E0=mix(E0,le.zw,ke);}if((i0&Zf)!=0u){d0 U8=I1(uintBitsToFloat(K0(PB,m0*4u)));c me=N0(U8,-2.*A0+E0+w0);c ne=N0(U8,-2.*E0+L0+A0);float k1=max(dot(me,me),dot(ne,ne));float P3=max(ceil(sqrt(.75*4.*sqrt(k1))),1.);R2=min(uint(P3),R2);}uint e9=R2+ie+m2-1u;d0 J2=X9(w0,A0,E0,L0);float e1=acos(W9(J2[0],J2[1]));float o4=e1/float(ie);float mb=determinant(d0(E0-w0,L0-A0));if(mb==.0)mb=determinant(J2);if(mb<.0)o4=-o4;A6=f(w0,A0);B6=f(E0,L0);O4=f(float(e9)-abs(d9-q2.x),float(e9),(m2<<10)|R2,o4);P4.xy=VC.xy;if(m2>1u){d0 nb=d0(J2[1],VC.xy);float Zh=acos(W9(nb[0],nb[1]));float oe=float(m2);if((i0&(c4|A8))==(y8|A8)){oe-=2.;}float ob=Zh/oe;if(determinant(nb)<.0)ob=-ob;P4.z=ob;}if(d9<c9){i0|=H3;}I7=i0;f W=p8(q2,2./Wf,l.Cd);
#ifdef SC
W.y=-W.y;
#endif
a0(A6);a0(B6);a0(O4);a0(P4);a0(I7);z1(W);}
#endif
#ifdef FB
F3 G3 c3(D4,HG){r(A6,f);r(B6,f);r(O4,f);r(P4,Q);r(I7,uint);c w0=A6.xy;c A0=A6.zw;c E0=B6.xy;c L0=B6.zw;d0 J2=X9(w0,A0,E0,L0);float ai=max(floor(O4.x),.0);float e9=O4.y;uint pe=uint(O4.z);float R2=float(pe&0x3ffu);float m2=float(pe>>10);float o4=O4.w;uint i0=I7;float Q4=e9-m2;float U1=ai;if(U1<=Q4){i0&=~c4;}else{w0=A0=E0=L0;J2=d0(J2[1],P4.xy);R2=1.;U1-=Q4;Q4=m2;o4=P4.z;if((i0&c4)>y8){if(U1<2.5)i0|=ua;if(U1>1.5&&U1<3.5)i0|=Sc;}else if((i0&A8)!=0u||(i0&c4)==z8){Q4-=2.;--U1;}i0|=o4<.0?B8:Tc;}c E5;float e1=.0;if(U1==.0||U1==Q4||(i0&c4)>y8){bool J8=U1<Q4*.5;E5=J8?w0:L0;e1=Cc(J8?J2[0]:J2[1]);}else if((i0&Rc)!=0u){E5=w0;if(U1>=float(ra/2u))E5=A0;if(U1>=float(ra*3u/4u))E5=E0;if(U1>=float(ra*7u/8u))E5=P4.xy;}else{float q1,F5;if(R2==Q4){q1=U1/R2;F5=.0;}else{c C,H,k2=A0-w0;c P6=L0-w0;c i8=E0-A0;H=i8-k2;C=-3.*i8+P6;c bi=H*(R2*2.);c R6=k2*(R2*R2);float f9=.0;float ci=min(R2-1.,U1);c pb=normalize(J2[0]);float di=-abs(o4);float ei=(1.+U1)*abs(o4);for(int qb=Th-1;qb>=0;--qb){float K7=f9+exp2(float(qb));if(K7<=ci){c rb=K7*C+bi;rb=K7*rb+R6;float fi=dot(normalize(rb),pb);float sb=K7*di+ei;sb=min(sb,E3);if(fi>=cos(sb))f9=K7;}}float gi=f9/R2;float qe=U1-f9;float g9=acos(clamp(pb.x,-1.,1.));g9=pb.y>=.0?g9:-g9;e1=qe*o4+g9;c Z2=c(sin(e1),-cos(e1));float m=dot(Z2,C),h9=dot(Z2,H),G1=dot(Z2,k2);float hi=max(h9*h9-m*G1,.0);float w2=sqrt(hi);if(h9>.0)w2=-w2;w2-=h9;float re=-.5*w2*m;c tb=(abs(w2*w2+re)<abs(m*G1+re))?c(w2,m):c(G1,w2);F5=(tb.y!=.0)?tb.x/tb.y:.0;F5=clamp(F5,.0,1.);if(qe==.0)F5=.0;q1=max(gi,F5);}c ii=c6(w0,A0,q1);c se=c6(A0,E0,q1);c ji=c6(E0,L0,q1);c te=c6(ii,se,q1);c ue=c6(se,ji,q1);E5=c6(te,ue,q1);if(q1!=F5)e1=Cc(ue-te);}D4 L7;L7.xy=da(E5);if((i0&c4)==z8){L7.z=ea((uint(Q4)<<16)|uint(U1));}else{L7.z=da(mod(e1,q8));}L7.w=ea(i0);K2(L7);}
#endif
