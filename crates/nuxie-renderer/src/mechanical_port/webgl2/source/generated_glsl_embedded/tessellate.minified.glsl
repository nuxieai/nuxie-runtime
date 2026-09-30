#define Uh 10
#ifdef CB
h1(g0)J(0,f,LD);J(1,f,MD);J(2,f,UC);
#ifdef ba
J(3,uint,HE);J(4,uint,IE);J(5,uint,JE);J(6,uint,KE);
#else
J(3,Y,SB);
#endif
i1
#endif
q2 I0 W(0,f,C6);I0 W(1,f,D6);I0 W(2,f,R4);I0 W(3,R,S4);T2 W(4,uint,J7);i2
#ifdef CB
X3 k6(f3,j7,XC);Y3 f4(j7,ga)E4 M4(Zc,Eg,OB);M4(ad,Fg,HD);F4 A1(FG,g0,F,B,v){K(v,F,LD,f);K(v,F,MD,f);K(v,F,UC,f);
#ifdef ba
K(v,F,HE,uint);K(v,F,IE,uint);K(v,F,JE,uint);K(v,F,KE,uint);Y SB=Y(HE,IE,JE,KE);
#else
K(v,F,SB,Y);
#endif
U(C6,f);U(D6,f);U(R4,f);U(S4,R);U(J7,uint);c x0=LD.xy;c B0=LD.zw;c F0=MD.xy;c M0=MD.zw;bool he=B<4;float y=he?UC.z:UC.w;int lb=int(he?SB.x:SB.y);
#ifdef Bc
int ie=lb<<16;if(SB.z==0xffffffffu){--ie;}float e9=float(ie>>16);
#else
float e9=float(lb<<16>>16);
#endif
float f9=float(lb>>16);c r2=c((B&1)==0?e9:f9,(B&2)==0?y+1.:y);if((f9-e9)*j.Dd<.0){r2.y=2.*y+1.-r2.y;}uint S2=SB.z&0x3ffu;uint je=(SB.z>>10)&0x3ffu;uint n2=SB.z>>20;uint j0=SB.w;uint I8=j0&Vc;uint n0=I8>0u?L0(HD,max(I8,1u)-1u).z:0u;Y O4=n0!=0u?L0(OB,n0*4u+1u):Y(0u,0u,0u,0u);float M2=uintBitsToFloat(O4.z);float N2=uintBitsToFloat(O4.w);if(N2!=.0&&M2==.0){float ke;float Vh=Cf(x0,B0,F0,M0,ke);float mb=N2*(1./ua);float Wh=xf(x0,B0,F0,M0,ke,mb);float K7=1.-Wh*(1./H3);float Xh=dot(M0-x0,M0-x0)/(mb*mb);float Yh=(Xh-1.)*.5;K7=min(K7,Yh);K7=min(K7,.99);float Zh=.5*K7;float x=Ac(Zh)*-2.+1.;float le=n8(x*N2,Vh);f me=mix(x0.xyxy,M0.xyxy,f(1./3.,1./3.,2./3.,2./3.));B0=mix(B0,me.xy,le);F0=mix(F0,me.zw,le);}if((j0&ag)!=0u){e0 W8=K1(uintBitsToFloat(L0(OB,n0*4u)));c ne=O0(W8,-2.*B0+F0+x0);c oe=O0(W8,-2.*F0+M0+B0);float m1=max(dot(ne,ne),dot(oe,oe));float S3=max(ceil(sqrt(.75*4.*sqrt(m1))),1.);S2=min(uint(S3),S2);}uint g9=S2+je+n2-1u;e0 K2=Y9(x0,B0,F0,M0);float f1=acos(X9(K2[0],K2[1]));float q4=f1/float(je);float nb=determinant(e0(F0-x0,M0-B0));if(nb==.0)nb=determinant(K2);if(nb<.0)q4=-q4;C6=f(x0,B0);D6=f(F0,M0);R4=f(float(g9)-abs(f9-r2.x),float(g9),(n2<<10)|S2,q4);S4.xy=UC.xy;if(n2>1u){e0 ob=e0(K2[1],UC.xy);float ai=acos(X9(ob[0],ob[1]));float pe=float(n2);if((j0&(e4|C8))==(A8|C8)){pe-=2.;}float pb=ai/pe;if(determinant(ob)<.0)pb=-pb;S4.z=pb;}if(f9<e9){j0|=K3;}J7=j0;f X=r8(r2,2./Xf,j.Dd);
#ifdef RC
X.y=-X.y;
#endif
c0(C6);c0(D6);c0(R4);c0(S4);c0(J7);B1(X);}
#endif
#ifdef EB
I3 J3 d3(G4,GG){r(C6,f);r(D6,f);r(R4,f);r(S4,R);r(J7,uint);c x0=C6.xy;c B0=C6.zw;c F0=D6.xy;c M0=D6.zw;e0 K2=Y9(x0,B0,F0,M0);float bi=max(floor(R4.x),.0);float g9=R4.y;uint qe=uint(R4.z);float S2=float(qe&0x3ffu);float n2=float(qe>>10);float q4=R4.w;uint j0=J7;float T4=g9-n2;float W1=bi;if(W1<=T4){j0&=~e4;}else{x0=B0=F0=M0;K2=e0(K2[1],S4.xy);S2=1.;W1-=T4;T4=n2;q4=S4.z;if((j0&e4)>A8){if(W1<2.5)j0|=va;if(W1>1.5&&W1<3.5)j0|=Tc;}else if((j0&C8)!=0u||(j0&e4)==B8){T4-=2.;--W1;}j0|=q4<.0?D8:Uc;}c I5;float f1=.0;if(W1==.0||W1==T4||(j0&e4)>A8){bool L8=W1<T4*.5;I5=L8?x0:M0;f1=Dc(L8?K2[0]:K2[1]);}else if((j0&Sc)!=0u){I5=x0;if(W1>=float(sa/2u))I5=B0;if(W1>=float(sa*3u/4u))I5=F0;if(W1>=float(sa*7u/8u))I5=S4.xy;}else{float v1,J5;if(S2==T4){v1=W1/S2;J5=.0;}else{c C,H,l2=B0-x0;c Q6=M0-x0;c k8=F0-B0;H=k8-l2;C=-3.*k8+Q6;c ci=H*(S2*2.);c S6=l2*(S2*S2);float h9=.0;float di=min(S2-1.,W1);c qb=normalize(K2[0]);float ei=-abs(q4);float fi=(1.+W1)*abs(q4);for(int rb=Uh-1;rb>=0;--rb){float L7=h9+exp2(float(rb));if(L7<=di){c sb=L7*C+ci;sb=L7*sb+S6;float gi=dot(normalize(sb),qb);float tb=L7*ei+fi;tb=min(tb,H3);if(gi>=cos(tb))h9=L7;}}float hi=h9/S2;float re=W1-h9;float i9=acos(clamp(qb.x,-1.,1.));i9=qb.y>=.0?i9:-i9;f1=re*q4+i9;c a3=c(sin(f1),-cos(f1));float n=dot(a3,C),j9=dot(a3,H),I1=dot(a3,l2);float ii=max(j9*j9-n*I1,.0);float x2=sqrt(ii);if(j9>.0)x2=-x2;x2-=j9;float se=-.5*x2*n;c ub=(abs(x2*x2+se)<abs(n*I1+se))?c(x2,n):c(I1,x2);J5=(ub.y!=.0)?ub.x/ub.y:.0;J5=clamp(J5,.0,1.);if(re==.0)J5=.0;v1=max(hi,J5);}c ji=e6(x0,B0,v1);c te=e6(B0,F0,v1);c ki=e6(F0,M0,v1);c ue=e6(ji,te,v1);c ve=e6(te,ki,v1);I5=e6(ue,ve,v1);if(v1!=J5)f1=Dc(ve-ue);}G4 M7;M7.xy=ea(I5);if((j0&e4)==B8){M7.z=fa((uint(T4)<<16)|uint(W1));}else{M7.z=ea(mod(f1,v8));}M7.w=fa(j0);L2(M7);}
#endif
