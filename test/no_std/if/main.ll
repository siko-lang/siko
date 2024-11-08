%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_condFalse(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_condTrue(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_final(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b12i2 = alloca %struct.siko_Tuple_, align 4
   %b12i1 = alloca %struct.siko_Tuple_, align 4
   %b10i2 = alloca %struct.siko_Tuple_, align 4
   %b10i1 = alloca %struct.siko_Tuple_, align 4
   %b7i3 = alloca %struct.siko_Tuple_, align 4
   %b7i2 = alloca %struct.siko_Tuple_, align 4
   %b7i1 = alloca %struct.siko_Tuple_, align 4
   %b6i2 = alloca %struct.siko_Tuple_, align 4
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_1 = alloca %struct.siko_Tuple_, align 4
   %b1i2 = alloca %struct.Bool_Bool, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %b0i1)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @Bool_Bool_True(ptr %b1i2)
   br label %block8
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 0, label %block5
]

block3:
   br label %block4
block4:
   call void @siko_Tuple_(ptr %b4i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i1, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @Main_condTrue(ptr %b6i1)
   call void @siko_Tuple_(ptr %b6i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i2, i64 0, i1 false)
   br label %block1
block7:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b7i1, ptr align 4 %match_var_1, i64 0, i1 false)
   call void @Main_final(ptr %b7i2)
   call void @siko_Tuple_(ptr %b7i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b7i3, i64 0, i1 false)
   ret void
block8:
   %tmp_switch_var_block8_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b1i2, i32 0, i32 0
   %tmp_switch_var_block8_2 = load i32, ptr %tmp_switch_var_block8_1, align 4
   switch i32 %tmp_switch_var_block8_2, label %block9 [
i32 0, label %block11
]

block9:
   br label %block10
block10:
   call void @Main_condFalse(ptr %b10i1)
   call void @siko_Tuple_(ptr %b10i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_1, ptr align 4 %b10i2, i64 0, i1 false)
   br label %block7
block11:
   br label %block12
block12:
   call void @Main_condTrue(ptr %b12i1)
   call void @siko_Tuple_(ptr %b12i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_1, ptr align 4 %b12i2, i64 0, i1 false)
   br label %block7
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


